use crate::state::AppState;
use crate::presentation::http::handlers::user_privacy;
use axum::routing::{get, patch, post, put};
use axum::Router;

/// Create user privacy routes
pub fn user_privacy_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/:user_id/privacy",
            get(user_privacy::get_privacy_settings),
        )
        .route(
            "/:user_id/privacy/dm",
            put(user_privacy::update_dm_privacy),
        )
        .route(
            "/:user_id/privacy/friend-requests",
            put(user_privacy::update_friend_request_privacy),
        )
        .route(
            "/:user_id/privacy/visibility",
            patch(user_privacy::update_visibility),
        )
        .route(
            "/:user_id/privacy/content",
            patch(user_privacy::update_content_settings),
        )
        .route(
            "/:user_id/privacy/preset",
            post(user_privacy::apply_preset),
        )
}
