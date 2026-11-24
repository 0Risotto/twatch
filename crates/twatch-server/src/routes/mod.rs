//! REST API routes.

mod covers;
mod downloads;
mod dto;
mod history;
mod player;
mod search;
mod session;
mod settings;

pub use covers::covers;
pub use downloads::*;
pub use dto::*;
pub use history::*;
pub use player::*;
pub use search::*;
pub use session::*;
pub use settings::*;

use axum::Json;

pub async fn health() -> Json<Health> {
    Json(Health { ok: true, version: env!("CARGO_PKG_VERSION") })
}
