use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use crate::state::AppState;
use crate::presentation::http::handlers::UserProfileHandler;

/// Create user profile routes
pub fn user_profile_routes() -> Router<AppState> {
    Router::new()
        // Profile management
        .route("/:user_id", get(UserProfileHandler::get_profile))
        .route(
            "/username/:username",
            get(UserProfileHandler::get_profile_by_username),
        )
        .route("/:user_id", post(UserProfileHandler::create_profile))
        .route(
            "/:user_id",
            patch(UserProfileHandler::update_profile),
        )
        .route(
            "/:user_id/username",
            put(UserProfileHandler::change_username),
        )
        .route(
            "/:user_id",
            delete(UserProfileHandler::delete_profile),
        )
        // Presence
        .route(
            "/:user_id/status",
            put(UserProfileHandler::update_status),
        )
        .route(
            "/:user_id/custom-status",
            put(UserProfileHandler::set_custom_status),
        )
        .route(
            "/:user_id/custom-status",
            delete(UserProfileHandler::clear_custom_status),
        )
        // Search & Discovery
        .route("/search", get(UserProfileHandler::search_users))
        .route(
            "/online",
            get(UserProfileHandler::get_online_users),
        )
        .route(
            "/check-username/:username",
            get(UserProfileHandler::check_username_availability),
        )
}
