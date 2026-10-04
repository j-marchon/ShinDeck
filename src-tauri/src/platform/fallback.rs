use std::path::{Path, PathBuf};

use image::{RgbImage, RgbaImage};

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

pub fn default_clips_folder() -> PathBuf {
    home().join("Videos")
}

pub fn init_worker_thread() {}

pub fn video_thumbnail(_path: &Path, _max_width: u32) -> Option<RgbImage> {
    None
}

pub fn file_icon(_path: &Path, _size: u32) -> Option<RgbaImage> {
    None
}

pub fn shortcut_dirs() -> Vec<PathBuf> {
    vec![home().join(".local/share/applications")]
}

pub fn steam_root() -> Option<PathBuf> {
    Some(home().join(".steam/steam")).filter(|p| p.is_dir())
}

pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(path);
    let program = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    std::process::Command::new(program).arg(dir).spawn().map(|_| ())
}

/// Development fallback: uses the desktop trash where a command exists.
pub fn move_to_trash(path: &Path) -> std::io::Result<()> {
    let status = if cfg!(target_os = "macos") {
        std::process::Command::new("osascript")
            .args(["-e", "on run argv\ntell application \"Finder\" to delete (POSIX file (item 1 of argv))\nend run"])
            .arg(path)
            .status()?
    } else {
        std::process::Command::new("gio").arg("trash").arg(path).status()?
    };
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("The clip could not be moved to the trash"))
    }
}

pub fn style_window(_window: &tauri::WebviewWindow) {}
