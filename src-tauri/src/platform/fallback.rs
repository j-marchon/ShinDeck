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

pub fn style_window(_window: &tauri::WebviewWindow) {}
