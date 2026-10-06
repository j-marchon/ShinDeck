//! Persistent user data: settings, favorites, the "edited in ShinDeck" marks
//! and the per-game merge counters.
//!
//! All live as small JSON files in the per-user app config directory
//! (`%APPDATA%\com.shindeck.app` on Windows) and are written atomically so a
//! crash can never leave a half-written file behind.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// What to do with the result of an edit (trim, cut, compress).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SaveMode {
    /// Ask after every edit.
    Ask,
    /// Keep the original and save the result next to it.
    New,
    /// Overwrite the original clip.
    Replace,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Root folder that holds one sub-folder per captured game.
    pub library_path: Option<PathBuf>,
    /// `None` until the user has made a choice the first time they edit.
    pub save_mode: Option<SaveMode>,
    /// Set when the user opted out of the "move to Recycle Bin?" prompt.
    /// Stored inverted so a missing key (older settings files) keeps asking.
    pub skip_delete_confirm: bool,
    /// Whether a merge replaces its originals. `None` until the user picks,
    /// in which case it follows the edit save mode.
    pub merge_replace: Option<bool>,
}

impl Settings {
    pub fn setup_complete(&self) -> bool {
        self.library_path.is_some()
    }

    pub fn merge_replace(&self) -> bool {
        self.merge_replace.unwrap_or(self.save_mode == Some(SaveMode::Replace))
    }
}

/// A set of clip ids (paths relative to the library root), used for favorites
/// and edited marks. Relative ids survive moving the whole library.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdSet(BTreeSet<String>);

impl IdSet {
    /// Adds or removes an id. Returns whether anything changed.
    pub fn set(&mut self, id: &str, present: bool) -> bool {
        if present {
            self.0.insert(id.to_owned())
        } else {
            self.0.remove(id)
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.0.contains(id)
    }

    /// Moves a mark from one id to another (after a rename). Returns whether
    /// anything changed.
    pub fn rename(&mut self, from: &str, to: &str) -> bool {
        if from != to && self.0.remove(from) {
            self.0.insert(to.to_owned());
            true
        } else {
            false
        }
    }
}

/// How many merges were made per game (keyed by game id), which numbers the
/// merged files: "Valorant Merge #3".
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MergeCounts(BTreeMap<String, u32>);

impl MergeCounts {
    pub fn get(&self, game: &str) -> u32 {
        self.0.get(game).copied().unwrap_or(0)
    }

    pub fn set(&mut self, game: &str, count: u32) {
        self.0.insert(game.to_owned(), count);
    }
}

/// A JSON-backed value bound to a file on disk.
pub struct Store<T> {
    path: PathBuf,
    pub value: T,
}

impl<T: Serialize + DeserializeOwned + Default> Store<T> {
    /// Loads the file, falling back to `T::default()` when it is missing or
    /// unreadable (a corrupt file should never prevent the app from starting).
    pub fn load(path: PathBuf) -> Self {
        let value = fs::read(&path)
            .ok()
            .and_then(|bytes| match serde_json::from_slice(&bytes) {
                Ok(v) => Some(v),
                Err(e) => {
                    log::warn!("ignoring unreadable {}: {e}", path.display());
                    None
                }
            })
            .unwrap_or_default();
        Self { path, value }
    }

    pub fn save(&self) -> Result<()> {
        write_atomic(&self.path, &serde_json::to_vec_pretty(&self.value)?)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
