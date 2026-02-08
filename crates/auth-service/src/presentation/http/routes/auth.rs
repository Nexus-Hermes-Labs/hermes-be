use crate::presentation::http::handlers;
use crate::presentation::http::state::AppState;
use axum::{routing::post, Router};

/// Public authentication routes (no token required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::auth::register_handler))
        .route("/login", post(handlers::auth::login_handler))
        .route("/refresh", post(handlers::auth::refresh_token_handler))
        .route("/logout", post(handlers::auth::logout_handler))
}
