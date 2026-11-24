use axum::Json;
use axum::extract::{Query, State};
use twatch_core::domain::search::SearchConfig;

use super::dto::{SearchConfigDto, SearchParams};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<twatch_core::domain::search::SearchResult>>, ApiError> {
    if params.q.trim().is_empty() {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(state.gateway.search(params.q.trim(), params.page).await?))
}

pub async fn get_search_config(State(state): State<AppState>) -> Json<SearchConfigDto> {
    Json(state.gateway.search_config().into())
}

pub async fn put_search_config(
    State(state): State<AppState>,
    Json(config): Json<SearchConfigDto>,
) -> Result<Json<SearchConfigDto>, ApiError> {
    let config: SearchConfig = config.into();
    state.gateway.update_search_config(config.clone())?;
    Ok(Json(config.into()))
}
