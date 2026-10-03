//! Commands invoked from the frontend (`src/lib/api`).

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::{Error, Result};
use crate::library::{self, FolderSummary, Library};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    library_path: Option<PathBuf>,
    setup_complete: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> SettingsView {
    let settings = &state.settings.lock().unwrap().value;
    SettingsView {
        library_path: settings.library_path.clone(),
        setup_complete: settings.setup_complete(),
    }
}

#[tauri::command]
pub fn default_clips_folder() -> PathBuf {
    crate::platform::default_clips_folder()
}

#[tauri::command]
pub async fn inspect_folder(path: PathBuf) -> FolderSummary {
    tauri::async_runtime::spawn_blocking(move || library::inspect_folder(&path))
        .await
        .unwrap_or(FolderSummary { exists: false, game_count: 0, clip_count: 0 })
}

#[tauri::command]
pub fn set_library_path(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<SettingsView> {
    if !path.is_dir() {
        return Err(Error::Message(format!("\"{}\" is not a folder", path.display())));
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings.value.library_path = Some(path.clone());
        settings.save()?;
    }
    activate_library(&app, &path);
    Ok(get_settings(state))
}

/// Grants the webview access to the library's videos and starts watching it.
pub fn activate_library(app: &AppHandle, root: &Path) {
    if let Err(e) = app.asset_protocol_scope().allow_directory(root, true) {
        log::error!("cannot allow {} in asset scope: {e}", root.display());
    }
    let watcher = crate::watcher::watch(app.clone(), root);
    *app.state::<AppState>().watcher.lock().unwrap() = watcher;
}

#[tauri::command]
pub async fn scan_library(app: AppHandle) -> Result<Library> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let root = state
            .settings
            .lock()
            .unwrap()
            .value
            .library_path
            .clone()
            .ok_or_else(|| Error::Message("No clips folder configured".into()))?;

        let library = {
            let favorites = state.favorites.lock().unwrap();
            let index = state.index.read().unwrap();
            library::scan(&root, &favorites.value, &index)?
        };
        state.index.write().unwrap().replace(&library);
        Ok(library)
    })
    .await
    .map_err(|e| Error::Message(e.to_string()))?
}

#[tauri::command]
pub fn set_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> Result<()> {
    let mut favorites = state.favorites.lock().unwrap();
    if favorites.value.set(&id, favorite) {
        favorites.save()?;
    }
    Ok(())
}

#[tauri::command]
pub fn reveal_clip(state: State<'_, AppState>, id: String) -> Result<()> {
    let path = state
        .index
        .read()
        .unwrap()
        .get(&id)
        .map(|e| e.path.clone())
        .ok_or_else(|| Error::Message("Clip not found".into()))?;
    crate::platform::reveal_in_file_manager(&path)?;
    Ok(())
}
