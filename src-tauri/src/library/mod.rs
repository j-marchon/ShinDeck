//! The clip library: discovering clips on disk and grouping them by game.

mod mp4;
mod scanner;

pub use scanner::{clip_at, inspect_folder, is_video, scan, FolderSummary, Marks};

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

/// Name used for clips that sit directly in the library root instead of a
/// game sub-folder.
pub const UNSORTED_GAME: &str = "";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    /// Stable identifier: the path relative to the library root, using `/`.
    pub id: String,
    /// File name without extension, as displayed in the gallery.
    pub name: String,
    /// Absolute path, used by the frontend to stream the video.
    pub path: PathBuf,
    /// Id of the game (the sub-folder name) this clip belongs to.
    pub game: String,
    pub size: u64,
    /// Capture time in milliseconds since the Unix epoch.
    pub date: u64,
    /// Last modification time (ms), used to invalidate cached thumbnails.
    pub modified: u64,
    pub duration_ms: Option<u64>,
    pub favorite: bool,
    /// Produced or modified by ShinDeck's editor.
    pub edited: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// The folder name ShadowPlay created for the game.
    pub id: String,
    pub name: String,
    pub clip_count: usize,
    /// Capture time of the newest clip (ms), handy for "recently played".
    pub latest: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub root: PathBuf,
    pub games: Vec<Game>,
    pub clips: Vec<Clip>,
}

/// Fast id -> file lookup used by the media protocols. Only files that were
/// discovered by a scan can be served, which also prevents path traversal.
#[derive(Default)]
pub struct ClipIndex {
    entries: HashMap<String, IndexEntry>,
    /// Durations survive rescans so unchanged files are never parsed twice.
    durations: HashMap<String, (u64, u64, Option<u64>)>,
}

#[derive(Clone)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}

impl ClipIndex {
    pub fn get(&self, id: &str) -> Option<&IndexEntry> {
        self.entries.get(id)
    }

    pub(crate) fn cached_duration(
        &self,
        id: &str,
        size: u64,
        modified: u64,
    ) -> Option<Option<u64>> {
        match self.durations.get(id) {
            Some(&(s, m, d)) if s == size && m == modified => Some(d),
            _ => None,
        }
    }

    /// Registers (or refreshes) a single clip, e.g. after a rename or export.
    pub(crate) fn upsert(&mut self, clip: &Clip) {
        self.entries.insert(
            clip.id.clone(),
            IndexEntry { path: clip.path.clone(), size: clip.size, modified: clip.modified },
        );
        self.durations.insert(clip.id.clone(), (clip.size, clip.modified, clip.duration_ms));
    }

    pub(crate) fn remove(&mut self, id: &str) {
        self.entries.remove(id);
        self.durations.remove(id);
    }

    pub(crate) fn replace(&mut self, library: &Library) {
        self.entries.clear();
        self.durations.clear();
        for clip in &library.clips {
            self.upsert(clip);
        }
    }
}

/// Turns a folder name into a display name.
pub fn display_game_name(id: &str) -> String {
    if id == UNSORTED_GAME {
        "Unsorted".to_owned()
    } else {
        id.to_owned()
    }
}
