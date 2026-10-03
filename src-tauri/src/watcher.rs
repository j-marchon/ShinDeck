//! Watches the clips folder so new recordings appear without a manual refresh.

use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use tauri::{AppHandle, Emitter};

use crate::library::is_video;

pub const LIBRARY_CHANGED_EVENT: &str = "library-changed";

/// ShadowPlay writes a clip over a second or two; waiting for the burst of
/// events to settle avoids rescanning mid-write.
const DEBOUNCE: Duration = Duration::from_millis(1500);

pub fn watch(app: AppHandle, root: &Path) -> Option<Debouncer<RecommendedWatcher>> {
    let mut debouncer = new_debouncer(DEBOUNCE, move |result: DebounceEventResult| {
        let Ok(events) = result else { return };
        // Videos, or extension-less paths (folders being added/removed/renamed).
        let relevant = events.iter().any(|e| is_video(&e.path) || e.path.extension().is_none());
        if relevant {
            let _ = app.emit(LIBRARY_CHANGED_EVENT, ());
        }
    })
    .map_err(|e| log::warn!("cannot create folder watcher: {e}"))
    .ok()?;

    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .map_err(|e| log::warn!("cannot watch {}: {e}", root.display()))
        .ok()?;
    Some(debouncer)
}
