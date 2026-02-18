use axum::Router;
use axum::routing::{delete, get, post, put};
use crate::state::AppState;
use crate::presentation::http::handlers::user_relationship;

/// Create user relationship routes
pub fn user_relationship_routes() -> Router<AppState> {
    Router::new()
        // Friend Requests
        .route(
            "/:user_id/relationships/request",
            post(user_relationship::send_friend_request),
        )
        .route(
            "/:user_id/relationships/request/accept",
            put(user_relationship::accept_friend_request),
        )
        .route(
            "/:user_id/relationships/request/decline",
            put(user_relationship::decline_friend_request),
        )
        
        // Friends
        .route(
            "/:user_id/relationships/friend/:target_user_id",
            delete(user_relationship::remove_friend),
        )
        
        // Blocks
        .route(
            "/:user_id/relationships/block",
            post(user_relationship::block_user),
        )
        .route(
            "/:user_id/relationships/block/:target_user_id",
            delete(user_relationship::unblock_user),
        )
        
        // Specific queries first
        .route(
            "/:user_id/relationships/friends",
            get(user_relationship::get_friends),
        )
        .route(
            "/:user_id/relationships/incoming",
            get(user_relationship::get_incoming_requests),
        )
        .route(
            "/:user_id/relationships/outgoing",
            get(user_relationship::get_outgoing_requests),
        )
        .route(
            "/:user_id/relationships/blocked",
            get(user_relationship::get_blocked_users),
        )
        
        // Generic query last
        .route(
            "/:user_id/relationships/:target_user_id",
            get(user_relationship::get_relationship),
        )
}
