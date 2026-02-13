use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use crate::application::UserProfileService;
use crate::domain::user_profile::UserProfileRepository;
use crate::presentation::UserProfileHandler;

/// Create user profile routes
pub fn user_profile_routes() -> Router<AppState>
{
    Router::new()
        // Profile management
        .route("/users/:user_id", get(UserProfileHandler::get_profile))
        .route(
            "/users/username/:username",
            get(UserProfileHandler::get_profile_by_username),
        )
        .route("/users/:user_id", post(UserProfileHandler::create_profile))
        .route(
            "/users/:user_id",
            patch(UserProfileHandler::update_profile),
        )
        .route(
            "/users/:user_id/username",
            put(UserProfileHandler::change_username),
        )
        .route(
            "/users/:user_id",
            delete(UserProfileHandler::delete_profile),
        )
        // Presence
        .route(
            "/users/:user_id/status",
            put(UserProfileHandler::update_status),
        )
        .route(
            "/users/:user_id/custom-status",
            put(UserProfileHandler::set_custom_status),
        )
        .route(
            "/users/:user_id/custom-status",
            delete(UserProfileHandler::clear_custom_status),
        )
        // Search & Discovery
        .route("/users/search", get(UserProfileHandler::search_users))
        .route(
            "/users/online",
            get(UserProfileHandler::get_online_users),
        )
        .route(
            "/users/check-username/:username",
            get(UserProfileHandler::check_username_availability),
        )
}