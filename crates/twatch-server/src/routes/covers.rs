use axum::Json;
use axum::extract::State;
use twatch_engine::covers::{Cover, CoverQuery};

use super::dto::CoverBatchRequest;
use crate::state::AppState;

pub async fn covers(
    State(state): State<AppState>,
    Json(request): Json<CoverBatchRequest>,
) -> Json<Vec<Cover>> {
    let queries: Vec<CoverQuery> = request
        .items
        .into_iter()
        .take(60)
        .map(|item| CoverQuery { title: item.title, kind: item.kind })
        .collect();
    Json(state.covers.resolve_many(&queries).await)
}
