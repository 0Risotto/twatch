use axum::Json;
use axum::extract::State;

use twatch_core::gateway::{BootstrapSnapshot, DeleteReport, DownloadSnapshot, SessionSnapshot};

use super::dto::{
    DeleteFilesRequest, DownloadRequest, OkResponse, PreviewRequest, RenameFileRequest,
    SelectFilesRequest, WatchRequest,
};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn bootstrap(State(state): State<AppState>) -> Json<BootstrapSnapshot> {
    Json(state.gateway.bootstrap())
}

pub async fn session(State(state): State<AppState>) -> Json<Option<SessionSnapshot>> {
    Json(state.gateway.session())
}

pub async fn preview(
    State(state): State<AppState>,
    Json(request): Json<PreviewRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    let url = request.url.trim();
    if url.is_empty() {
        return Err(ApiError::bad_request("A magnet link or torrent URL is required"));
    }
    Ok(Json(state.gateway.preview(url).await?))
}

pub async fn close_session(State(state): State<AppState>) -> Json<OkResponse> {
    state.gateway.close_session();
    Json(OkResponse { ok: true })
}

pub async fn select_files(
    State(state): State<AppState>,
    Json(request): Json<SelectFilesRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    state.gateway.select_files(&request.indices, request.selected)?;
    Ok(Json(state.gateway.session().ok_or_else(|| ApiError::bad_request("No active session"))?))
}

pub async fn watch_file(
    State(state): State<AppState>,
    Json(request): Json<WatchRequest>,
) -> Result<Json<DownloadSnapshot>, ApiError> {
    Ok(Json(state.gateway.watch_file(request.index)?))
}

pub async fn download_files(
    State(state): State<AppState>,
    Json(request): Json<DownloadRequest>,
) -> Result<Json<DownloadSnapshot>, ApiError> {
    Ok(Json(state.gateway.download_files(&request.indices)?))
}

pub async fn rename_file(
    State(state): State<AppState>,
    Json(request): Json<RenameFileRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    state.gateway.rename_file(request.index, &request.name)?;
    Ok(Json(state.gateway.session().ok_or_else(|| ApiError::bad_request("No active session"))?))
}

pub async fn delete_files(
    State(state): State<AppState>,
    Json(request): Json<DeleteFilesRequest>,
) -> Json<DeleteReport> {
    Json(state.gateway.delete_files(&request.indices))
}
