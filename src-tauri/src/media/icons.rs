//! Game icon resolution.
//!
//! ShadowPlay only gives us a folder name per game ("Counter-strike 2"), not
//! the executable, so icons are found by matching that name against things
//! that *do* know where the game lives:
//!
//! 1. Start menu and desktop shortcuts (`.lnk`, and Steam's `.url` files).
//!    Launchers like Steam, Epic, Battle.net, EA and Riot create these, and
//!    the shell resolves them to the game's own high-resolution icon.
//! 2. Steam's library cache: `appmanifest_*.acf` files map game names to app
//!    ids, and `appcache/librarycache` holds the icon Steam shows for each.
//!
//! Names are compared after normalisation (case, punctuation, ™/®), first
//! exactly and then by containment. Results are cached as PNG on disk; when
//! nothing matches, the frontend draws a monogram instead.

use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use image::{ImageFormat, RgbaImage};

use super::{stable_hash, Worker};

const ICON_SIZE: u32 = 64;
const MAX_SHORTCUT_DEPTH: usize = 4;
/// Shortcuts whose names contain these are never the game itself.
const JUNK_WORDS: &[&str] = &[
    "uninstall",
    "readme",
    "read me",
    "manual",
    "website",
    "support",
    "help",
    "documentation",
    "eula",
    "license",
    "changelog",
    "release notes",
    "redistributable",
];

#[derive(Debug, Clone)]
enum Source {
    /// Ask the shell for the icon of this file (.lnk, .url, .exe, .ico).
    Shell(PathBuf),
    /// A ready-made image file (Steam's cached icons).
    Image(PathBuf),
}

struct Candidate {
    key: String,
    source: Source,
}

#[derive(Default)]
struct IconIndex {
    shortcuts: Vec<Candidate>,
    steam: Vec<Candidate>,
}

pub struct GameIcons {
    dir: PathBuf,
    worker: Worker,
    index: Arc<OnceLock<IconIndex>>,
    /// Games without any icon this session, so we do not search again.
    misses: Arc<Mutex<HashSet<String>>>,
}

impl GameIcons {
    pub fn new(dir: PathBuf, worker: Worker) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir, worker, index: Default::default(), misses: Default::default() }
    }

    fn cache_path(&self, game: &str) -> PathBuf {
        self.dir.join(format!("{:016x}.png", stable_hash(&[game.as_bytes()])))
    }

    /// Calls `done` with PNG bytes, or `None` when no icon could be found.
    pub fn get(&self, game: String, done: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
        let cache = self.cache_path(&game);
        if let Ok(bytes) = fs::read(&cache) {
            return done(Some(bytes));
        }
        if game.is_empty() || self.misses.lock().unwrap().contains(&game) {
            return done(None);
        }
        let index = self.index.clone();
        let misses = self.misses.clone();
        self.worker.spawn(move || {
            let index = index.get_or_init(IconIndex::build);
            let bytes = index.resolve(&game).find_map(|source| load(&source)).and_then(encode);
            match &bytes {
                Some(bytes) => {
                    let _ = fs::write(&cache, bytes);
                }
                None => {
                    misses.lock().unwrap().insert(game);
                }
            }
            done(bytes)
        });
    }
}

fn load(source: &Source) -> Option<RgbaImage> {
    match source {
        Source::Shell(path) => crate::platform::file_icon(path, ICON_SIZE),
        Source::Image(path) => image::open(path).ok().map(|i| i.into_rgba8()),
    }
}

fn encode(image: RgbaImage) -> Option<Vec<u8>> {
    let image = if image.width() > ICON_SIZE * 2 {
        image::imageops::thumbnail(&image, ICON_SIZE * 2, ICON_SIZE * 2)
    } else {
        image
    };
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

/// Lowercase alphanumerics only: "Counter-Strike® 2" -> "counterstrike2".
pub fn normalize(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

impl IconIndex {
    fn build() -> Self {
        let mut index = IconIndex::default();
        for dir in crate::platform::shortcut_dirs() {
            collect_shortcuts(&dir, 0, &mut index.shortcuts);
        }
        if let Some(root) = crate::platform::steam_root() {
            index.steam = steam_icons(&root);
        }
        log::info!(
            "icon index: {} shortcuts, {} steam games",
            index.shortcuts.len(),
            index.steam.len()
        );
        index
    }

    /// Icon sources for a game, best first.
    fn resolve<'a>(&'a self, game: &str) -> impl Iterator<Item = Source> + 'a {
        let key = normalize(game);
        let all = || self.shortcuts.iter().chain(&self.steam);
        let exact: Vec<Source> = all().filter(|c| c.key == key).map(|c| c.source.clone()).collect();
        let fuzzy = if exact.is_empty() { fuzzy_matches(&key, all()) } else { Vec::new() };
        exact.into_iter().chain(fuzzy)
    }
}

/// Containment matching ("Battlefield 2042" vs "Battlefield™ 2042 Open Beta"),
/// accepted only when the names are of similar length to avoid "Halo"
/// matching "Halo Infinite Multiplayer Editor", and ordered by closeness.
fn fuzzy_matches<'a>(key: &str, candidates: impl Iterator<Item = &'a Candidate>) -> Vec<Source> {
    if key.chars().count() < 4 {
        return Vec::new();
    }
    let mut scored: Vec<(f32, &Candidate)> = candidates
        .filter_map(|c| {
            let (short, long) = if c.key.len() <= key.len() {
                (c.key.as_str(), key)
            } else {
                (key, c.key.as_str())
            };
            if short.len() < 4 || !long.contains(short) {
                return None;
            }
            let ratio = short.len() as f32 / long.len() as f32;
            (ratio >= 0.6).then_some((ratio, c))
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    scored.into_iter().map(|(_, c)| c.source.clone()).collect()
}

fn collect_shortcuts(dir: &Path, depth: usize, out: &mut Vec<Candidate>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else { continue };
        if file_type.is_dir() {
            if depth < MAX_SHORTCUT_DEPTH {
                collect_shortcuts(&path, depth + 1, out);
            }
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
        let source = match ext.as_deref() {
            Some("lnk") => Source::Shell(path.clone()),
            Some("url") => internet_shortcut_icon(&path).unwrap_or(Source::Shell(path.clone())),
            _ => continue,
        };
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let lower = stem.to_lowercase();
        if JUNK_WORDS.iter().any(|w| lower.contains(w)) {
            continue;
        }
        out.push(Candidate { key: normalize(stem), source });
    }
}

/// Steam's `.url` shortcuts point `IconFile=` at a high-resolution `.ico`.
fn internet_shortcut_icon(path: &Path) -> Option<Source> {
    let text = fs::read_to_string(path).ok()?;
    let icon = text.lines().find_map(|l| l.trim().strip_prefix("IconFile="))?;
    let icon = PathBuf::from(icon.trim());
    icon.is_file().then_some(Source::Shell(icon))
}

// ---------------------------------------------------------------------------
// Steam
// ---------------------------------------------------------------------------

/// Extracts `"key"  "value"` pairs from Valve's KeyValues text format. Nesting
/// is ignored, which is enough for the flat fields we need.
fn vdf_pairs(text: &str) -> impl Iterator<Item = (String, String)> + '_ {
    text.lines().filter_map(|line| {
        let mut parts = line.split('"').skip(1).step_by(2);
        let key = parts.next()?;
        let value = parts.next()?;
        Some((key.to_ascii_lowercase(), value.replace("\\\\", "\\")))
    })
}

fn steam_libraries(root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![root.to_path_buf()];
    if let Ok(text) = fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")) {
        for (key, value) in vdf_pairs(&text) {
            let path = PathBuf::from(value);
            if key == "path" && !libraries.contains(&path) {
                libraries.push(path);
            }
        }
    }
    libraries
}

fn steam_icons(root: &Path) -> Vec<Candidate> {
    let cache = root.join("appcache").join("librarycache");
    let mut out = Vec::new();
    for library in steam_libraries(root) {
        let Ok(entries) = fs::read_dir(library.join("steamapps")) else { continue };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            let Ok(text) = fs::read_to_string(entry.path()) else { continue };
            let (mut app_id, mut game) = (None, None);
            for (key, value) in vdf_pairs(&text) {
                match key.as_str() {
                    "appid" if app_id.is_none() => app_id = Some(value),
                    "name" if game.is_none() => game = Some(value),
                    _ => {}
                }
            }
            if let (Some(app_id), Some(game)) = (app_id, game) {
                if let Some(icon) = steam_cached_icon(&cache, &app_id) {
                    out.push(Candidate { key: normalize(&game), source: Source::Image(icon) });
                }
            }
        }
    }
    out
}

/// Older clients store `<appid>_icon.jpg`; newer ones use a per-app folder
/// in which the icon is the image named after its SHA-1 hash.
fn steam_cached_icon(cache: &Path, app_id: &str) -> Option<PathBuf> {
    let legacy = cache.join(format!("{app_id}_icon.jpg"));
    if legacy.is_file() {
        return Some(legacy);
    }
    fs::read_dir(cache.join(app_id)).ok()?.flatten().map(|e| e.path()).find(|p| {
        let is_hash = p
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()));
        let is_jpg = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jpg"));
        is_hash && is_jpg
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(name: &str) -> Candidate {
        Candidate { key: normalize(name), source: Source::Shell(PathBuf::from(name)) }
    }

    fn names(sources: Vec<Source>) -> Vec<String> {
        sources
            .into_iter()
            .map(|s| match s {
                Source::Shell(p) | Source::Image(p) => p.to_string_lossy().into_owned(),
            })
            .collect()
    }

    #[test]
    fn normalizes_trademarks_and_case() {
        assert_eq!(normalize("Counter-Strike® 2"), normalize("Counter-strike 2"));
        assert_eq!(normalize("Tom Clancy's Rainbow Six® Siege"), "tomclancysrainbowsixsiege");
    }

    #[test]
    fn exact_match_wins() {
        let index = IconIndex {
            shortcuts: vec![candidate("Valorant"), candidate("Counter-Strike 2")],
            steam: vec![],
        };
        assert_eq!(names(index.resolve("Counter-strike 2").collect()), ["Counter-Strike 2"]);
    }

    #[test]
    fn fuzzy_requires_similar_length() {
        let pool = [candidate("Battlefield™ 2042"), candidate("Halo Infinite Multiplayer Editor")];
        assert_eq!(
            names(fuzzy_matches(&normalize("Battlefield 2042 Beta"), pool.iter())),
            ["Battlefield™ 2042"]
        );
        assert!(fuzzy_matches(&normalize("Halo"), pool.iter()).is_empty());
    }

    #[test]
    fn parses_vdf_pairs() {
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"730\"\n\t\"name\"\t\t\"Counter-Strike 2\"\n\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n}";
        let pairs: Vec<_> = vdf_pairs(text).collect();
        assert!(pairs.contains(&("appid".into(), "730".into())));
        assert!(pairs.contains(&("path".into(), "D:\\SteamLibrary".into())));
    }
}
