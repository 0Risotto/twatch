use axum::Json;
use axum::extract::State;

use twatch_core::gateway::LibraryEntry;

use crate::state::AppState;

pub async fn library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.library())
}

pub async fn rescan_library(State(state): State<AppState>) -> Json<Vec<LibraryEntry>> {
    Json(state.gateway.rescan_library())
}
