use std::collections::HashMap;
use std::fs::{self, DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rayon::prelude::*;
use serde::Serialize;

use super::{display_game_name, mp4, Clip, ClipIndex, Game, Library, UNSORTED_GAME};
use crate::config::Favorites;
use crate::error::{Error, Result};

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov", "mkv", "webm"];

/// How deep to look inside a game folder. ShadowPlay writes clips directly
/// into it, but a little slack keeps hand-organised sub-folders visible.
const MAX_GAME_DEPTH: usize = 3;

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO_EXTENSIONS.iter().any(|v| v.eq_ignore_ascii_case(e)))
}

/// A video file found on disk, before metadata enrichment.
struct Found {
    path: PathBuf,
    game: String,
    meta: Metadata,
}

/// Top level of the library: every sub-folder is a game, loose videos are
/// "unsorted".
struct RootListing {
    games: Vec<(String, PathBuf)>,
    loose: Vec<Found>,
}

fn read_root(root: &Path) -> Result<RootListing> {
    let entries = fs::read_dir(root).map_err(|e| {
        Error::Message(format!("Cannot open clips folder \"{}\": {e}", root.display()))
    })?;

    let mut games = Vec::new();
    let mut loose = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if file_type.is_dir() {
            games.push((name, entry.path()));
        } else if let Some(found) = as_video(&entry, UNSORTED_GAME) {
            loose.push(found);
        }
    }
    Ok(RootListing { games, loose })
}

fn as_video(entry: &DirEntry, game: &str) -> Option<Found> {
    let path = entry.path();
    if !is_video(&path) {
        return None;
    }
    // On Windows the metadata comes straight from the directory listing, so
    // this does not touch the file itself.
    let meta = entry.metadata().ok()?;
    meta.is_file().then(|| Found { path, game: game.to_owned(), meta })
}

fn collect_videos(dir: &Path, game: &str, depth: usize, out: &mut Vec<Found>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        if file_type.is_dir() {
            if depth + 1 < MAX_GAME_DEPTH {
                collect_videos(&entry.path(), game, depth + 1, out);
            }
        } else if let Some(found) = as_video(&entry, game) {
            out.push(found);
        }
    }
}

fn find_all(root: &Path) -> Result<Vec<Found>> {
    let RootListing { games, loose: mut found } = read_root(root)?;
    let per_game: Vec<Vec<Found>> = games
        .par_iter()
        .map(|(name, dir)| {
            let mut out = Vec::new();
            collect_videos(dir, name, 0, &mut out);
            out
        })
        .collect();
    found.extend(per_game.into_iter().flatten());
    Ok(found)
}

fn millis(time: std::io::Result<std::time::SystemTime>) -> Option<u64> {
    time.ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
}

fn clip_id(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}

/// Scans the whole library. Durations are reused from `previous` for files
/// that did not change, so rescans only parse new clips.
pub fn scan(root: &Path, favorites: &Favorites, previous: &ClipIndex) -> Result<Library> {
    let found = find_all(root)?;

    let clips: Vec<Clip> = found
        .into_par_iter()
        .map(|f| {
            let id = clip_id(root, &f.path);
            let modified = millis(f.meta.modified()).unwrap_or(0);
            // A copied file gets a fresh creation time but keeps its
            // modification time; the earlier of the two is the capture time.
            let date = millis(f.meta.created()).map_or(modified, |c| c.min(modified));
            let size = f.meta.len();
            let duration_ms = previous
                .cached_duration(&id, size, modified)
                .unwrap_or_else(|| mp4::duration_ms(&f.path));
            let name =
                f.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            Clip {
                favorite: favorites.contains(&id),
                id,
                name,
                path: f.path,
                game: f.game,
                size,
                date,
                modified,
                duration_ms,
            }
        })
        .collect();

    let mut games: HashMap<&str, Game> = HashMap::new();
    for clip in &clips {
        let game = games.entry(&clip.game).or_insert_with(|| Game {
            id: clip.game.clone(),
            name: display_game_name(&clip.game),
            clip_count: 0,
            latest: 0,
        });
        game.clip_count += 1;
        game.latest = game.latest.max(clip.date);
    }
    let mut games: Vec<Game> = games.into_values().collect();
    games.sort_by_cached_key(|g| (g.id == UNSORTED_GAME, g.name.to_lowercase()));

    Ok(Library { root: root.to_path_buf(), games, clips })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSummary {
    pub exists: bool,
    pub game_count: usize,
    pub clip_count: usize,
}

/// Cheap preview used by the setup screen while the user picks a folder.
pub fn inspect_folder(root: &Path) -> FolderSummary {
    if !root.is_dir() {
        return FolderSummary { exists: false, game_count: 0, clip_count: 0 };
    }
    match find_all(root) {
        Ok(found) => {
            let mut games: Vec<&str> = found.iter().map(|f| f.game.as_str()).collect();
            games.sort_unstable();
            games.dedup();
            FolderSummary { exists: true, game_count: games.len(), clip_count: found.len() }
        }
        Err(_) => FolderSummary { exists: true, game_count: 0, clip_count: 0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_clips_by_game_folder() {
        let root = std::env::temp_dir().join(format!("shindeck-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (path, contents) in [
            ("Counter-strike 2/Counter-strike 2 2025.01.02 - 20.00.00.01.DVR.mp4", "x"),
            ("Counter-strike 2/screenshot.png", "x"),
            ("Valorant/highlights/Valorant 2025.01.03.mp4", "x"),
            ("loose clip.MP4", "x"),
            ("Empty Game/notes.txt", "x"),
        ] {
            let file = root.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, contents).unwrap();
        }

        let mut favorites = Favorites::default();
        favorites.set("Valorant/highlights/Valorant 2025.01.03.mp4", true);
        let library = scan(&root, &favorites, &ClipIndex::default()).unwrap();
        let _ = fs::remove_dir_all(&root);

        let mut clips: Vec<(&str, &str, bool)> =
            library.clips.iter().map(|c| (c.game.as_str(), c.id.as_str(), c.favorite)).collect();
        clips.sort();
        assert_eq!(
            clips,
            [
                ("", "loose clip.MP4", false),
                (
                    "Counter-strike 2",
                    "Counter-strike 2/Counter-strike 2 2025.01.02 - 20.00.00.01.DVR.mp4",
                    false
                ),
                ("Valorant", "Valorant/highlights/Valorant 2025.01.03.mp4", true),
            ]
        );
        let games: Vec<&str> = library.games.iter().map(|g| g.name.as_str()).collect();
        // Folders without clips are hidden; loose clips come last.
        assert_eq!(games, ["Counter-strike 2", "Valorant", "Unsorted"]);
    }
}
