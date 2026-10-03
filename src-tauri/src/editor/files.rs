//! File-system helpers for renames and exports.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::error::{Error, Result};

const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const RESERVED: &[&str] = &["CON", "PRN", "AUX", "NUL"];
const MAX_NAME_LEN: usize = 180;

/// Validates a new clip name (without extension) against Windows' rules.
pub fn validate_name(name: &str) -> Result<&str> {
    let name = name.trim();
    let invalid = |why: &str| Err(Error::Message(why.to_owned()));
    if name.is_empty() {
        return invalid("The name can't be empty");
    }
    if name.chars().count() > MAX_NAME_LEN {
        return invalid("That name is too long");
    }
    if let Some(c) = name.chars().find(|c| INVALID_CHARS.contains(c) || c.is_control()) {
        return Err(Error::Message(format!("Names can't contain {c}")));
    }
    if name.ends_with('.') {
        return invalid("Names can't end with a dot");
    }
    if name.starts_with('.') {
        return invalid("Names can't start with a dot");
    }
    let upper = name.to_ascii_uppercase();
    let device = upper.split('.').next().unwrap_or_default();
    let numbered = (device.starts_with("COM") || device.starts_with("LPT"))
        && device.len() == 4
        && device.as_bytes()[3].is_ascii_digit();
    if RESERVED.contains(&device) || numbered {
        return invalid("That name is reserved by Windows");
    }
    Ok(name)
}

/// `dir/stem.ext`, or `dir/stem (2).ext`, `(3)`… if taken.
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let candidate = dir.join(format!("{stem}.{ext}"));
    if !candidate.exists() {
        return candidate;
    }
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}).{ext}")))
        .find(|p| !p.exists())
        .expect("unbounded search")
}

/// Hidden work file next to `near`. The leading dot keeps it out of the
/// gallery while it is being written.
pub fn temp_path(near: &Path, tag: &str) -> PathBuf {
    let stem = near.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    near.with_file_name(format!(".{stem}.shindeck-{tag}-{}.mp4", std::process::id()))
}

/// The webview may still hold a clip open for a moment (it streams with
/// range requests), which makes Windows refuse renames; retry briefly.
fn retry<T>(mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut attempts = 0;
    loop {
        match op() {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempts < 25 => {
                attempts += 1;
                thread::sleep(Duration::from_millis(120));
            }
            result => return result,
        }
    }
}

pub fn rename(from: &Path, to: &Path) -> Result<()> {
    retry(|| fs::rename(from, to)).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => {
            Error::Message("The file is in use by another program".into())
        }
        _ => e.into(),
    })
}

pub fn remove(path: &Path) -> Result<()> {
    Ok(retry(|| fs::remove_file(path))?)
}

/// Swaps `replacement` in for `original`, keeping the original as a backup
/// until the swap succeeded so a failure never loses the clip.
pub fn replace(original: &Path, replacement: &Path) -> Result<()> {
    let backup = temp_path(original, "backup");
    rename(original, &backup)?;
    if let Err(e) = rename(replacement, original) {
        let _ = rename(&backup, original);
        return Err(e);
    }
    let _ = remove(&backup);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_windows_names() {
        assert_eq!(validate_name("  Ace clutch  ").unwrap(), "Ace clutch");
        for bad in
            ["", "   ", "a/b", "what?", "dots.", ".hidden", "CON", "com1", "LPT9.clip", "a\u{7}"]
        {
            assert!(validate_name(bad).is_err(), "{bad:?} should be rejected");
        }
        assert!(validate_name("Console highlights").is_ok());
        assert!(validate_name("Valorant 2026.08.21 - 18.58.45.11.DVR").is_ok());
    }

    #[test]
    fn finds_unique_paths() {
        let dir = std::env::temp_dir().join(format!("shindeck-files-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("clip (trimmed).mp4"), "x").unwrap();
        fs::write(dir.join("clip (trimmed) (2).mp4"), "x").unwrap();
        assert_eq!(unique_path(&dir, "clip (trimmed)", "mp4"), dir.join("clip (trimmed) (3).mp4"));
        assert_eq!(unique_path(&dir, "other", "mp4"), dir.join("other.mp4"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
