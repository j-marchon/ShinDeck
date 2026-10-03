//! Persistent user data: settings and favorites.
//!
//! Both live as small JSON files in the per-user app config directory
//! (`%APPDATA%\com.shindeck.app` on Windows) and are written atomically so a
//! crash can never leave a half-written file behind.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Root folder that holds one sub-folder per captured game.
    pub library_path: Option<PathBuf>,
}

impl Settings {
    pub fn setup_complete(&self) -> bool {
        self.library_path.is_some()
    }
}

/// Favorites are keyed by clip id (path relative to the library root), so they
/// survive moving the whole library to another drive.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Favorites(BTreeSet<String>);

impl Favorites {
    pub fn set(&mut self, id: &str, favorite: bool) -> bool {
        if favorite {
            self.0.insert(id.to_owned())
        } else {
            self.0.remove(id)
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.0.contains(id)
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
