//! Windows integration through the Shell APIs.
//!
//! Thumbnails and icons come from `IShellItemImageFactory`, the same API
//! Explorer uses. That means:
//! * no bundled video decoder (the app stays tiny),
//! * thumbnails are served from the system thumbnail cache when Explorer has
//!   already generated them, which makes them nearly free,
//! * any codec the system can decode (H.264, HEVC with the extension, AV1…)
//!   produces a thumbnail.

use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use image::{RgbImage, RgbaImage};
use windows::core::{GUID, HSTRING};
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY, RRF_RT_REG_SZ,
};
use windows::Win32::UI::Shell::{
    FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_Programs, FOLDERID_PublicDesktop,
    FOLDERID_Videos, IShellItemImageFactory, SHCreateItemFromParsingName, SHGetKnownFolderPath,
    KF_FLAG_DEFAULT, SIIGBF, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, SIIGBF_THUMBNAILONLY,
};

/// Each worker thread hosts shell extensions (thumbnail providers), which
/// expect a single-threaded COM apartment.
pub fn init_worker_thread() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    }
}

// ---------------------------------------------------------------------------
// Folders
// ---------------------------------------------------------------------------

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let path = raw.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(raw.0 as *const c_void));
        path
    }
}

/// Reads a string from `HKEY_CURRENT_USER`, accepting both `REG_SZ` and the
/// UTF-16 `REG_BINARY` blobs NVIDIA uses for its paths.
fn read_user_registry_string(key: &str, value: &str) -> Option<String> {
    let key = HSTRING::from(key);
    let value = HSTRING::from(value);
    let flags = RRF_RT_REG_SZ | RRF_RT_REG_BINARY;
    unsafe {
        let mut len = 0u32;
        RegGetValueW(HKEY_CURRENT_USER, &key, &value, flags, None, None, Some(&mut len))
            .ok()
            .ok()?;
        let mut buf = vec![0u16; (len as usize).div_ceil(2)];
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &value,
            flags,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut len),
        )
        .ok()
        .ok()?;
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let s = String::from_utf16_lossy(&buf[..end]);
        (!s.trim().is_empty()).then(|| s.trim().to_owned())
    }
}

/// Where ShadowPlay saves clips:
/// 1. the path configured in GeForce Experience / the NVIDIA app,
/// 2. `Videos\NVIDIA` (NVIDIA app default) if it exists,
/// 3. the user's Videos folder (GeForce Experience default).
pub fn default_clips_folder() -> PathBuf {
    if let Some(configured) = read_user_registry_string(
        r"Software\NVIDIA Corporation\Global\ShadowPlay\NVSPCAPS",
        "DefaultPathW",
    )
    .map(PathBuf::from)
    .filter(|p| p.is_dir())
    {
        return configured;
    }

    let videos = known_folder(&FOLDERID_Videos).unwrap_or_else(|| {
        std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_default().join("Videos")
    });
    let nvidia = videos.join("NVIDIA");
    if nvidia.is_dir() {
        nvidia
    } else {
        videos
    }
}

/// Folders that contain game shortcuts (Start menu and desktops).
pub fn shortcut_dirs() -> Vec<PathBuf> {
    [&FOLDERID_Programs, &FOLDERID_CommonPrograms, &FOLDERID_Desktop, &FOLDERID_PublicDesktop]
        .into_iter()
        .filter_map(known_folder)
        .collect()
}

pub fn steam_root() -> Option<PathBuf> {
    read_user_registry_string(r"Software\Valve\Steam", "SteamPath")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(r"C:\Program Files (x86)\Steam")))
        .filter(|p| p.is_dir())
}

pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn()
        .map(|_| ())
}

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

/// Top-down BGRA pixels of a shell bitmap.
struct Bgra {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

fn shell_image(path: &Path, size: SIZE, flags: SIIGBF) -> Option<Bgra> {
    unsafe {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None).ok()?;
        let bitmap = factory.GetImage(size, flags).ok()?;
        let pixels = read_bitmap(bitmap);
        let _ = DeleteObject(bitmap.into());
        pixels
    }
}

unsafe fn read_bitmap(bitmap: HBITMAP) -> Option<Bgra> {
    let mut info = BITMAP::default();
    let read = GetObjectW(
        bitmap.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut info as *mut BITMAP as *mut c_void),
    );
    if read == 0 || info.bmWidth <= 0 || info.bmHeight == 0 {
        return None;
    }
    let width = info.bmWidth;
    let height = info.bmHeight.abs();

    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // negative = top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    let dc = CreateCompatibleDC(None);
    let lines = GetDIBits(
        dc,
        bitmap,
        0,
        height as u32,
        Some(pixels.as_mut_ptr().cast()),
        &mut header,
        DIB_RGB_COLORS,
    );
    let _ = DeleteDC(dc);
    (lines > 0).then_some(Bgra { width: width as u32, height: height as u32, pixels })
}

/// The video's thumbnail (never a generic file icon), at most `max_width` wide.
pub fn video_thumbnail(path: &Path, max_width: u32) -> Option<RgbImage> {
    let size = SIZE { cx: max_width as i32, cy: max_width as i32 };
    let bgra = shell_image(path, size, SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK)?;
    let rgb = bgra.pixels.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0]]).collect();
    RgbImage::from_raw(bgra.width, bgra.height, rgb)
}

/// The icon of an executable, shortcut (.lnk/.url) or .ico file.
pub fn file_icon(path: &Path, size: u32) -> Option<RgbaImage> {
    let size = SIZE { cx: size as i32, cy: size as i32 };
    let bgra = shell_image(path, size, SIIGBF_ICONONLY)?;
    let has_alpha = bgra.pixels.chunks_exact(4).any(|p| p[3] != 0);
    let rgba = bgra
        .pixels
        .chunks_exact(4)
        .flat_map(|p| {
            if !has_alpha {
                return [p[2], p[1], p[0], 255];
            }
            // Shell bitmaps use premultiplied alpha; PNG wants straight alpha.
            let a = p[3] as u32;
            let un =
                |c: u8| (c as u32 * 255 + a / 2).checked_div(a).map_or(0, |v| v.min(255) as u8);
            [un(p[2]), un(p[1]), un(p[0]), p[3]]
        })
        .collect();
    RgbaImage::from_raw(bgra.width, bgra.height, rgba)
}

// ---------------------------------------------------------------------------
// Window chrome
// ---------------------------------------------------------------------------

/// The window has no native title bar (the app draws its own). Windows 11
/// still draws a 1px frame around it, by default in the system accent colour;
/// tint it to match the app and keep the rounded corners.
pub fn style_window(window: &tauri::WebviewWindow) {
    let Ok(handle) = window.hwnd() else { return };
    // Tauri links a different `windows` crate version; rewrap the raw handle.
    let hwnd = windows::Win32::Foundation::HWND(handle.0);
    // COLORREF is 0x00BBGGRR: a dim NVIDIA green (#3a5c0a).
    let border: u32 = 0x000a_5c3a;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &border as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
        let corners = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corners as *const _ as *const c_void,
            std::mem::size_of_val(&corners) as u32,
        );
    }
}
