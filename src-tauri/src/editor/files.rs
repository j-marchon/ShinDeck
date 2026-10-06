//! File-system helpers for renames and exports.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

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

/// The next free `dir/{game} Merge #n.mp4` after `count` earlier merges.
/// Numbers already taken on disk are skipped, so a lost counter never
/// overwrites anything. Returns the number used and the path.
pub fn merge_path(dir: &Path, game: &str, count: u32) -> (u32, PathBuf) {
    (count + 1..)
        .map(|n| (n, dir.join(format!("{game} Merge #{n}.mp4"))))
        .find(|(_, p)| !p.exists())
        .expect("unbounded search")
}

/// `n` when `stem` is "{name} #n" (case-insensitive, like Windows names).
pub fn numbered(stem: &str, name: &str) -> Option<u32> {
    let prefix = format!("{name} #");
    let head = stem.get(..prefix.len())?;
    if head.to_lowercase() != prefix.to_lowercase() {
        return None;
    }
    let digits = &stem[prefix.len()..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
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

/// Timestamps to carry from a clip onto its edited version, so the gallery's
/// date (and the date filters) keep describing when it was recorded.
pub struct Stamps {
    // Only Windows lets a program set the creation time.
    #[cfg_attr(not(windows), allow(dead_code))]
    created: Option<SystemTime>,
    modified: Option<SystemTime>,
}

impl Stamps {
    pub fn of(path: &Path) -> Self {
        let meta = fs::metadata(path).ok();
        Self {
            created: meta.as_ref().and_then(|m| m.created().ok()),
            modified: meta.as_ref().and_then(|m| m.modified().ok()),
        }
    }

    /// Best effort: a clip that keeps a new date is not worth failing an edit for.
    pub fn apply(&self, path: &Path) {
        let mut times = fs::FileTimes::new();
        if let Some(t) = self.modified {
            times = times.set_modified(t);
        }
        #[cfg(windows)]
        if let Some(t) = self.created {
            use std::os::windows::fs::FileTimesExt;
            times = times.set_created(t);
        }
        let _ = retry(|| fs::OpenOptions::new().write(true).open(path)?.set_times(times));
    }
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
    fn carries_timestamps_over() {
        let dir = std::env::temp_dir().join(format!("shindeck-stamps-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (old, new) = (dir.join("old.mp4"), dir.join("new.mp4"));
        fs::write(&old, "x").unwrap();
        fs::write(&new, "y").unwrap();
        let past = SystemTime::now() - Duration::from_secs(40 * 86_400);
        let file = fs::OpenOptions::new().write(true).open(&old).unwrap();
        file.set_times(fs::FileTimes::new().set_modified(past)).unwrap();
        drop(file);

        Stamps::of(&old).apply(&new);
        let got = fs::metadata(&new).unwrap().modified().unwrap();
        let diff = got.duration_since(past).unwrap_or_else(|e| e.duration());
        assert!(diff < Duration::from_secs(2), "modified time was not carried over");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn numbers_merges_per_game() {
        let dir = std::env::temp_dir().join(format!("shindeck-merges-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(merge_path(&dir, "Valorant", 0), (1, dir.join("Valorant Merge #1.mp4")));
        fs::write(dir.join("Valorant Merge #3.mp4"), "x").unwrap();
        assert_eq!(merge_path(&dir, "Valorant", 2), (4, dir.join("Valorant Merge #4.mp4")));
        assert_eq!(merge_path(&dir, "Valorant", 4).0, 5);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reads_batch_numbers() {
        assert_eq!(numbered("Ace #3", "Ace"), Some(3));
        assert_eq!(numbered("ace #12", "Ace"), Some(12));
        assert_eq!(numbered("Ace #", "Ace"), None);
        assert_eq!(numbered("Ace #2b", "Ace"), None);
        assert_eq!(numbered("Aces #2", "Ace"), None);
        assert_eq!(numbered("Ace", "Ace"), None);
        assert_eq!(numbered("Élan #4", "élan"), Some(4));
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
