//! `thumb://` and `gameicon://` URI schemes.
//!
//! The frontend points plain `<img>` tags at these URLs, so the webview
//! handles lazy loading, decoding and memory, while the work behind each
//! request happens on background threads.

use percent_encoding::percent_decode_str;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::state::AppState;

pub const THUMBNAIL_SCHEME: &str = "thumb";
pub const GAME_ICON_SCHEME: &str = "gameicon";
pub const FILMSTRIP_SCHEME: &str = "filmstrip";

/// `/Counter-strike%202/clip.mp4` -> `Counter-strike 2/clip.mp4`
fn requested_key(request: &Request<Vec<u8>>) -> String {
    let path = request.uri().path().trim_start_matches('/');
    percent_decode_str(path).decode_utf8_lossy().into_owned()
}

fn respond(responder: UriSchemeResponder, content_type: &str, bytes: Option<Vec<u8>>) {
    let response = match bytes {
        Some(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(bytes),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CACHE_CONTROL, "no-store")
            .body(Vec::new()),
    };
    responder.respond(response.expect("valid response"));
}

pub fn thumbnail<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let id = requested_key(&request);
    // Requests arrive on the UI thread; never touch the disk there.
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let entry = state.index.read().unwrap().get(&id).cloned();
        match entry {
            Some(entry) => {
                state.thumbnails.get(entry, move |bytes| respond(responder, "image/jpeg", bytes))
            }
            None => respond(responder, "image/jpeg", None),
        }
    });
}

pub fn game_icon<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let game = requested_key(&request);
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .icons
            .get(game, move |bytes| respond(responder, "image/png", bytes));
    });
}

pub fn filmstrip<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let id = requested_key(&request);
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let entry = state.index.read().unwrap().get(&id).cloned();
        let bytes = entry.and_then(|e| state.filmstrips.get(&e));
        respond(responder, "image/jpeg", bytes);
    });
}
