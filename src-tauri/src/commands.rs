//! Commands invoked from the frontend (`src/lib/api`).

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::config::SaveMode;
use crate::editor::{self, plan::EditSpec, Destination};
use crate::error::{Error, Result};
use crate::library::{self, Clip, FolderSummary, Library, Marks};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    library_path: Option<PathBuf>,
    setup_complete: bool,
    save_mode: Option<SaveMode>,
    /// Whether removing a clip asks for confirmation first.
    confirm_delete: bool,
    /// Whether the editor can run (ffmpeg found).
    editing_available: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> SettingsView {
    let settings = &state.settings.lock().unwrap().value;
    SettingsView {
        library_path: settings.library_path.clone(),
        setup_complete: settings.setup_complete(),
        save_mode: settings.save_mode,
        confirm_delete: !settings.skip_delete_confirm,
        editing_available: editor::ffmpeg::path().is_some(),
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
            let edited = state.edited.lock().unwrap();
            let index = state.index.read().unwrap();
            let marks = Marks { favorites: &favorites.value, edited: &edited.value };
            library::scan(&root, &marks, &index)?
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

#[tauri::command]
pub fn set_save_mode(state: State<'_, AppState>, mode: SaveMode) -> Result<SettingsView> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.value.save_mode = Some(mode);
        settings.save()?;
    }
    Ok(get_settings(state))
}

#[tauri::command]
pub fn set_confirm_delete(state: State<'_, AppState>, confirm: bool) -> Result<SettingsView> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.value.skip_delete_confirm = !confirm;
        settings.save()?;
    }
    Ok(get_settings(state))
}

/// Moves a clip to the Recycle Bin and forgets everything ShinDeck knew about it.
#[tauri::command]
pub async fn delete_clip(app: AppHandle, id: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || editor::delete(&app.state::<AppState>(), &id))
        .await
        .map_err(|e| Error::Message(e.to_string()))?
}

#[tauri::command]
pub async fn rename_clip(app: AppHandle, id: String, name: String) -> Result<Clip> {
    tauri::async_runtime::spawn_blocking(move || {
        editor::rename(&app.state::<AppState>(), &id, &name)
    })
    .await
    .map_err(|e| Error::Message(e.to_string()))?
}

pub const EXPORT_PROGRESS_EVENT: &str = "export-progress";

/// Trims/cuts/compresses a clip. Progress (0..1) is emitted as
/// `export-progress` events while ffmpeg runs.
#[tauri::command]
pub async fn export_clip(
    app: AppHandle,
    id: String,
    spec: EditSpec,
    destination: Destination,
) -> Result<Clip> {
    tauri::async_runtime::spawn_blocking(move || {
        let emitter = app.clone();
        let progress = move |p: f64| {
            let _ = emitter.emit(EXPORT_PROGRESS_EVENT, p);
        };
        editor::export(&app.state::<AppState>(), &id, &spec, destination, &progress)
    })
    .await
    .map_err(|e| Error::Message(e.to_string()))?
}

#[tauri::command]
pub fn cancel_export(state: State<'_, AppState>) {
    state.editor.cancel();
}
