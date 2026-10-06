use std::path::Path;
use std::sync::{Mutex, RwLock};

use notify_debouncer_mini::{notify::RecommendedWatcher, Debouncer};

use crate::config::{IdSet, MergeCounts, Settings, Store};
use crate::editor::{Editor, Filmstrips};
use crate::library::ClipIndex;
use crate::media::{GameIcons, Thumbnails, Worker};

/// Shell extraction is I/O- and decoder-bound; a few threads keep the gallery
/// filling quickly without starving the rest of the system.
const SHELL_THREADS: usize = 4;

pub struct AppState {
    pub settings: Mutex<Store<Settings>>,
    pub favorites: Mutex<Store<IdSet>>,
    /// Clips produced or modified by the editor.
    pub edited: Mutex<Store<IdSet>>,
    /// Numbers merged clips per game.
    pub merges: Mutex<Store<MergeCounts>>,
    pub index: RwLock<ClipIndex>,
    pub thumbnails: Thumbnails,
    pub icons: GameIcons,
    pub filmstrips: Filmstrips,
    pub editor: Editor,
    pub watcher: Mutex<Option<Debouncer<RecommendedWatcher>>>,
}

impl AppState {
    pub fn new(config_dir: &Path, cache_dir: &Path) -> Self {
        let worker = Worker::new(SHELL_THREADS);
        Self {
            settings: Mutex::new(Store::load(config_dir.join("settings.json"))),
            favorites: Mutex::new(Store::load(config_dir.join("favorites.json"))),
            edited: Mutex::new(Store::load(config_dir.join("edited.json"))),
            merges: Mutex::new(Store::load(config_dir.join("merges.json"))),
            index: RwLock::default(),
            thumbnails: Thumbnails::new(cache_dir.join("thumbnails"), worker.clone()),
            icons: GameIcons::new(cache_dir.join("icons"), worker),
            filmstrips: Filmstrips::new(cache_dir.join("filmstrips")),
            editor: Editor::default(),
            watcher: Mutex::default(),
        }
    }
}
