use crate::application::UserPrivacyService;
use crate::domain::user_privacy::UserPrivacyRepository;
use crate::presentation::UserPrivacyHandler;
use axum::routing::{get, patch, post, put};
use axum::Router;

/// Create user privacy routes
pub fn user_privacy_routes<R>() -> Router<AppState> {
    Router::new()
        .route(
            "/users/:user_id/privacy",
            get(UserPrivacyHandler::get_privacy_settings),
        )
        .route(
            "/users/:user_id/privacy/dm",
            put(UserPrivacyHandler::update_dm_privacy),
        )
        .route(
            "/users/:user_id/privacy/friend-requests",
            put(UserPrivacyHandler::update_friend_request_privacy),
        )
        .route(
            "/users/:user_id/privacy/visibility",
            patch(UserPrivacyHandler::update_visibility),
        )
        .route(
            "/users/:user_id/privacy/content",
            patch(UserPrivacyHandler::update_content_settings),
        )
        .route(
            "/users/:user_id/privacy/preset",
            post(UserPrivacyHandler::apply_preset),
        )
}
