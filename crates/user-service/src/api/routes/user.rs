use crate::api::handlers;
use crate::api::state::AppState;
use axum::{routing::post, Router};

/// Public authentication routes (no token required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
}
