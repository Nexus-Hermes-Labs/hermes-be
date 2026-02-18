use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use crate::state::AppState;
use crate::presentation::http::handlers::user_profile;

/// Create user profile routes
pub fn user_profile_routes() -> Router<AppState> {
    Router::new()
        // Specific routes first to avoid conflict with /:user_id
        .route("/", post(user_profile::create_profile))
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
        
        // Generic /:user_id routes
        .route("/:user_id", get(user_profile::get_profile))
        .route("/:user_id", patch(user_profile::update_profile))
        .route("/:user_id", delete(user_profile::delete_profile))
        .route("/:user_id/username", put(user_profile::change_username))
        .route("/:user_id/status", put(user_profile::update_status))
        .route("/:user_id/custom-status", put(user_profile::set_custom_status))
        .route("/:user_id/custom-status", delete(user_profile::clear_custom_status))
}
