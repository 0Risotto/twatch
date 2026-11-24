use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;

use twatch_core::gateway::PlayerSnapshot;

use super::dto::{OkResponse, PlayerOpenRequest};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn player(State(state): State<AppState>) -> Json<PlayerSnapshot> {
    Json(state.gateway.player())
}

pub async fn open_player(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayerOpenRequest>,
) -> Result<Json<PlayerSnapshot>, ApiError> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:7878");
    let scheme =
        headers.get("x-forwarded-proto").and_then(|value| value.to_str().ok()).unwrap_or("http");
    let base_url = format!("{scheme}://{host}");
    Ok(Json(state.gateway.open_player(request.download_id, &base_url)?))
}

pub async fn stop_player(State(state): State<AppState>) -> Json<OkResponse> {
    state.gateway.stop_player();
    Json(OkResponse { ok: true })
}
