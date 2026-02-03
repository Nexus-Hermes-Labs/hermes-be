use crate::api::handlers;
use crate::api::state::AppState;
use axum::{routing::post, Router};
use axum::routing::{get, patch, put};

/// Protected user routes (JWT authentication required)
///
/// All handlers use AuthenticatedUser extractor to access JWT claims.
/// Routes should be wrapped with auth middleware at the router level.
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        // ─── Profile Queries ─────────────────────────────────────
        .route("/me", get(handlers::user::get_my_profile_handler))
        .route("/:id", get(handlers::user::get_user_profile_handler))
        .route(
            "/username/:username",
            get(handlers::user::get_user_by_username_handler),
        )
        // ─── Profile Management ──────────────────────────────────
        .route("/me/profile", patch(handlers::user::update_profile_handler))
        // ─── Privacy Settings ────────────────────────────────────
        .route(
            "/me/privacy",
            patch(handlers::user::update_privacy_settings_handler),
        )
        // ─── Custom Status ───────────────────────────────────────
        .route(
            "/me/custom-status",
            put(handlers::user::set_custom_status_handler)
                .delete(handlers::user::clear_custom_status_handler),
        )
        // ─── Search ──────────────────────────────────────────────
        .route("/search", get(handlers::user::search_users_handler))
}

// Optional: Public routes if any endpoints don't require auth
// pub fn public_routes() -> Router<AppState> {
//     Router::new()
//         .route("/search", get(handlers::user::search_users_handler))
// }