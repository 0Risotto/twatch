use axum::Json;
use axum::extract::State;

use twatch_core::gateway::ConfigView;
use twatch_core::theme::{self, ThemeSpec};

use super::dto::ConfigPatch;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn themes() -> Json<Vec<ThemeSpec>> {
    Json(theme::THEMES.to_vec())
}

pub async fn config(State(state): State<AppState>) -> Json<ConfigView> {
    Json(state.gateway.config_view())
}

pub async fn patch_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigPatch>,
) -> Result<Json<ConfigView>, ApiError> {
    if let Some(theme) = request.theme {
        state.gateway.update_theme(&theme)?;
    }
    Ok(Json(state.gateway.config_view()))
}
