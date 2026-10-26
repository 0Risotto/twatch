//! Rust HTTP/WebSocket gateway for twatch.
//!
//! The daemon owns the engine; this crate exposes it to the SPA over a
//! local-first API.  The SPA is static: Rust serves the built assets.

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;
pub mod static_files;
pub mod stream;
pub mod subtitles;
pub mod ws;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{delete, get, patch, post, put};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use twatch_core::gateway::Gateway;
use twatch_engine::covers::CoverService;
use twatch_engine::stream::StreamService;

use crate::state::{AppState, ServerConfig};

/// Build the complete application router.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/bootstrap", get(routes::bootstrap))
        .route("/api/session", get(routes::session))
        .route("/api/session/preview", post(routes::preview))
        .route("/api/session/close", post(routes::close_session))
        .route("/api/session/files/select", post(routes::select_files))
        .route("/api/session/files/watch", post(routes::watch_file))
        .route("/api/session/files/download", post(routes::download_files))
        .route("/api/session/files/rename", patch(routes::rename_file))
        .route("/api/session/files", delete(routes::delete_files))
        .route("/api/downloads", get(routes::downloads))
        .route("/api/downloads/{id}/pause", post(routes::pause_download))
        .route("/api/downloads/{id}/resume", post(routes::resume_download))
        .route("/api/downloads/{id}", delete(routes::remove_download))
        .route("/api/search", get(routes::search))
        .route("/api/search/config", get(routes::get_search_config))
        .route("/api/search/config", put(routes::put_search_config))
        .route("/api/history", get(routes::history))
        .route("/api/history", patch(routes::rename_history))
        .route("/api/history", delete(routes::delete_history))
        .route("/api/history/open", post(routes::open_history))
        .route("/api/player", get(routes::player))
        .route("/api/player/open", post(routes::open_player))
        .route("/api/player/stop", post(routes::stop_player))
        .route("/api/themes", get(routes::themes))
        .route("/api/config", get(routes::config))
        .route("/api/config", patch(routes::patch_config))
        .route("/api/covers", post(routes::covers))
        .route("/api/stream/{torrent_id}/{file_idx}", get(stream::stream_file))
        .route("/api/subtitles/{torrent_id}/{file_idx}", get(subtitles::subtitles))
        .route("/api/ws", get(ws::ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth::require_token));

    Router::new()
        .route("/api/health", get(routes::health))
        .merge(protected)
        .route("/", get(static_files::index))
        .route("/assets/{*path}", get(static_files::asset))
        .route("/favicon.svg", get(static_files::favicon))
        .route("/theme-boot.js", get(static_files::theme_boot))
        .route("/tv_ghost_logo.svg", get(static_files::logo))
        .fallback(static_files::fallback)
        .layer(middleware::from_fn(auth::security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve until the process is stopped.
pub async fn serve(
    listener: TcpListener,
    gateway: Arc<Gateway>,
    stream: StreamService,
    covers: Arc<CoverService>,
    config: ServerConfig,
) -> anyhow::Result<()> {
    let state = AppState::new(gateway, Some(stream), covers, &config);
    let app = router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
