mod commands;
mod config;
mod editor;
mod error;
mod library;
mod media;
mod platform;
mod state;
mod watcher;

use tauri::Manager;

use media::protocol;
use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch just brings the running window forward.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol(protocol::THUMBNAIL_SCHEME, protocol::thumbnail)
        .register_asynchronous_uri_scheme_protocol(protocol::GAME_ICON_SCHEME, protocol::game_icon)
        .register_asynchronous_uri_scheme_protocol(protocol::FILMSTRIP_SCHEME, protocol::filmstrip)
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let cache_dir = app.path().app_cache_dir()?;
            let state = AppState::new(&config_dir, &cache_dir);
            let library_path = state.settings.lock().unwrap().value.library_path.clone();
            app.manage(state);

            if let Some(window) = app.get_webview_window("main") {
                platform::style_window(&window);
            }

            if let Some(root) = library_path {
                commands::activate_library(app.handle(), &root);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::default_clips_folder,
            commands::inspect_folder,
            commands::set_library_path,
            commands::scan_library,
            commands::set_favorite,
            commands::reveal_clip,
            commands::set_save_mode,
            commands::set_confirm_delete,
            commands::delete_clip,
            commands::rename_clip,
            commands::export_clip,
            commands::merge_clips,
            commands::cancel_export,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ShinDeck");
}
