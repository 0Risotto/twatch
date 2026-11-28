use axum::Json;
use axum::extract::{Path, State};

use twatch_core::gateway::{DeleteReport, LibraryEntry};

use crate::error::ApiError;
use crate::state::AppState;

pub async fn library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.library())
}

pub async fn rescan_library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.rescan_library())
}

pub async fn delete_library(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteReport>, ApiError> {
    Ok(Json(state.gateway.delete_library_file(&id)?))
}
