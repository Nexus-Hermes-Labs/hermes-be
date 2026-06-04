use crate::presentation::http::handlers;
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Public authentication routes (no token required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::auth::register_handler))
        .route("/login", post(handlers::auth::login_handler))
        .route("/refresh", post(handlers::auth::refresh_token_handler))
        .route("/logout", post(handlers::auth::logout_handler))
        .route("/verify-email", get(handlers::auth::verify_email_handler))
        .route(
            "/forgot-password",
            post(handlers::auth::forgot_password_handler),
        )
        .route(
            "/reset-password",
            post(handlers::auth::reset_password_handler),
        )
        .route(
            "/change-password",
            post(handlers::auth::change_password_handler),
        )
}
