//! Static SPA serving with a dev-directory fallback and optional embedding.

use std::collections::HashMap;
use std::path::{Component, Path as FsPath, PathBuf};

use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::auth::{has_valid_token, is_authenticated, token_cookie};
use crate::state::AppState;

#[cfg(feature = "embed-web")]
mod embedded {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "web-dist"]
    pub struct Assets;
}

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

fn content_type(path: &str) -> &'static str {
    mime_guess::from_path(path).first_raw().unwrap_or("application/octet-stream")
}

fn cache_control(path: &str) -> &'static str {
    if path.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" }
}

fn safe_path(root: &FsPath, path: &str) -> Option<PathBuf> {
    let relative = FsPath::new(path);
    if relative.is_absolute() {
        return None;
    }
    if relative.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        return None;
    }
    let full = root.join(relative);
    Some(full)
}

fn missing_ui_page() -> Html<String> {
    Html(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>twatchd</title>\
         <style>body{font:15px system-ui;background:#0c0e14;color:#c0caf5;padding:3rem;line-height:1.6}\
         code{background:#1a1b26;padding:.15rem .4rem;border-radius:5px}max-width:760px</style></head>\
         <body><h1>twatch web UI is not built yet</h1>\
         <p>Run <code>cd web &amp;&amp; npm install &amp;&amp; npm run build</code>, then start \
         <code>twatchd --web-dir web/dist</code>.</p>\
         <p>For development, run <code>npm run dev</code> in <code>web/</code>; Vite proxies \
         <code>/api</code> to this daemon.</p></body></html>"
            .to_owned(),
    )
}

fn read_asset(state: &AppState, path: &str) -> Option<Vec<u8>> {
    if let Some(root) = &state.web_dir {
        let full = safe_path(root, path)?;
        return std::fs::read(full).ok();
    }

    #[cfg(feature = "embed-web")]
    {
        embedded::Assets::get(path).map(|file| file.data.into_owned())
    }

    #[cfg(not(feature = "embed-web"))]
    {
        let _ = state;
        None
    }
}

fn read_index(state: &AppState) -> Option<Vec<u8>> {
    if let Some(root) = &state.web_dir {
        return std::fs::read(root.join("index.html")).ok();
    }

    #[cfg(feature = "embed-web")]
    {
        embedded::Assets::get("index.html").map(|file| file.data.into_owned())
    }

    #[cfg(not(feature = "embed-web"))]
    {
        let _ = state;
        None
    }
}

fn asset_response(path: &str, bytes: Vec<u8>) -> Response {
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type(path))
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_control(path)));
    response
}

fn index_response(state: &AppState, headers: &HeaderMap, query: &TokenQuery) -> Response {
    let local = !state.allow_remote;
    let query_ok = query.token.as_deref() == Some(state.token.as_ref());
    if !local && !query_ok && !has_valid_token(headers, &state.token) {
        return (
            StatusCode::UNAUTHORIZED,
            Html(
                "<!doctype html><html><body style=\"font:15px system-ui;background:#0c0e14;color:#c0caf5;padding:3rem\">\
                 <h1>twatchd</h1><p>Open the URL printed in your terminal, including its token.</p>\
                 </body></html>"
                    .to_owned(),
            ),
        )
            .into_response();
    }

    let Some(index) = read_index(state) else {
        return missing_ui_page().into_response();
    };

    let mut response = asset_response("index.html", index);
    if !is_authenticated(state, headers) || query_ok {
        response.headers_mut().append(header::SET_COOKIE, token_cookie(&state.token));
    }
    response
}

/// `/` — serve the SPA entry and install the auth cookie.
pub async fn index(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
) -> Response {
    index_response(&state, &headers, &query)
}

/// `/favicon.svg` — the public root asset.
pub async fn favicon(State(state): State<AppState>) -> Response {
    root_asset(&state, "favicon.svg")
}

/// `/theme-boot.js` — the no-flash theme bootstrap, kept external for CSP.
pub async fn theme_boot(State(state): State<AppState>) -> Response {
    root_asset(&state, "theme-boot.js")
}

/// `/tv_ghost_logo.svg` — the user-supplied brand mark.
pub async fn logo(State(state): State<AppState>) -> Response {
    root_asset(&state, "tv_ghost_logo.svg")
}

fn root_asset(state: &AppState, path: &str) -> Response {
    match read_asset(state, path) {
        Some(bytes) => asset_response(path, bytes),
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

/// `/assets/*` and any other static file under the web root.
pub async fn asset(
    State(state): State<AppState>,
    uri: Uri,
    Query(query): Query<TokenQuery>,
    headers: HeaderMap,
) -> Response {
    // Derive the path from the request URI instead of relying on the wildcard
    // capture shape: axum versions differ on whether the leading slash is
    // included, and a mistaken join silently turned every asset into a 404.
    let path = uri.path().trim_start_matches('/').to_owned();
    if path == "index.html" || path.is_empty() {
        return index_response(&state, &headers, &query);
    }
    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    match read_asset(&state, &path) {
        Some(bytes) => asset_response(&path, bytes),
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

/// SPA fallback: unknown non-API paths get the app shell so client-side routing
/// and deep links work.  Unknown `/api` paths remain JSON 404s.
pub async fn fallback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> Response {
    if request.uri().path().starts_with("/api/") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    let query = TokenQuery { token: query.get("token").cloned() };
    index_response(&state, &headers, &query)
}
