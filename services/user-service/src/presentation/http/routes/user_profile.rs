use crate::presentation::http::handlers::user_profile;
use crate::state::AppState;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

/// Read-only routes accessible to any authenticated user.
/// Allows looking up other users' profiles without admin privileges.
pub fn user_profile_read_routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(user_profile::search_users))
        .route("/online", get(user_profile::get_online_users))
        .route(
            "/username/:username",
            get(user_profile::get_profile_by_username),
        )
        .route(
            "/check-username/:username",
            get(user_profile::check_username_availability),
        )
        .route("/:user_id", get(user_profile::get_profile))
}

/// Admin-only routes for system-level user management operations.
pub fn user_profile_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(user_profile::create_profile))
        .route("/:user_id", patch(user_profile::update_profile))
        .route("/:user_id", delete(user_profile::delete_profile))
        .route("/:user_id/username", put(user_profile::change_username))
        .route("/:user_id/status", put(user_profile::update_status))
        .route(
            "/:user_id/custom-status",
            put(user_profile::set_custom_status),
        )
        .route(
            "/:user_id/custom-status",
            delete(user_profile::clear_custom_status),
        )
}
