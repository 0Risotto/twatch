use axum::Json;
use axum::extract::State;

use twatch_core::gateway::SessionSnapshot;

use super::dto::{
    HistoryEntryDto, HistoryRenameRequest, HistoryUrlRequest, OkResponse, ProgressRequest,
};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn history(State(state): State<AppState>) -> Json<Vec<HistoryEntryDto>> {
    Json(state.gateway.history().into_iter().map(HistoryEntryDto::from).collect())
}

pub async fn rename_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryRenameRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.rename_history(&request.url, &request.name)?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn delete_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryUrlRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.delete_history(&request.url)?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn open_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryUrlRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    Ok(Json(state.gateway.open_history(&request.url).await?))
}

pub async fn save_progress(
    State(state): State<AppState>,
    Json(request): Json<ProgressRequest>,
) -> Json<OkResponse> {
    state.gateway.save_progress(&request.url, &request.file, request.position, request.duration);
    Json(OkResponse { ok: true })
}
