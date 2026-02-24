use crate::presentation::http::handlers;
use crate::state::app_state::AppState;
use axum::{routing::any, Router};

/// Internal routes — not exposed through the public API prefix.
/// Used by Traefik's ForwardAuth middleware to validate JWT tokens.
pub fn internal_routes() -> Router<AppState> {
    Router::new().route("/verify", any(handlers::forward_auth::verify_handler))
}
