use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use twatch_core::gateway::{LibraryEntry, PlayerSnapshot};

use crate::error::ApiError;
use crate::state::AppState;

pub async fn library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.library())
}

pub async fn rescan_library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.rescan_library())
}

pub async fn open_library(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<PlayerSnapshot>, ApiError> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:7878");
    let scheme =
        headers.get("x-forwarded-proto").and_then(|value| value.to_str().ok()).unwrap_or("http");
    let base_url = format!("{scheme}://{host}");
    Ok(Json(state.gateway.open_library_file_with_token(
        &id,
        &base_url,
        Some(state.token.as_ref()),
    )?))
}
