use axum::Json;
use axum::extract::{Path, State};

use twatch_core::gateway::DownloadSnapshot;

use super::dto::OkResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn downloads(State(state): State<AppState>) -> Json<Vec<DownloadSnapshot>> {
    Json(state.gateway.downloads())
}

pub async fn pause_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.pause_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn resume_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.resume_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn remove_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.remove_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}
