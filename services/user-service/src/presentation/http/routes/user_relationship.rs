use axum::Router;
use axum::routing::{delete, get, post, put};
use crate::state::AppState;
use crate::presentation::http::handlers::UserRelationshipHandler;

/// Create user relationship routes
pub fn user_relationship_routes() -> Router<AppState> {
    Router::new()
        // Friend Requests
        .route(
            "/:user_id/relationships/request",
            post(UserRelationshipHandler::send_friend_request),
        )
        .route(
            "/:user_id/relationships/request/accept",
            put(UserRelationshipHandler::accept_friend_request),
        )
        .route(
            "/:user_id/relationships/request/decline",
            put(UserRelationshipHandler::decline_friend_request),
        )
        // Friends
        .route(
            "/:user_id/relationships/friend/:target_user_id",
            delete(UserRelationshipHandler::remove_friend),
        )
        // Blocks
        .route(
            "/:user_id/relationships/block",
            post(UserRelationshipHandler::block_user),
        )
        .route(
            "/:user_id/relationships/block/:target_user_id",
            delete(UserRelationshipHandler::unblock_user),
        )
        // Queries
        .route(
            "/:user_id/relationships/:target_user_id",
            get(UserRelationshipHandler::get_relationship),
        )
        .route(
            "/:user_id/relationships/friends",
            get(UserRelationshipHandler::get_friends),
        )
        .route(
            "/:user_id/relationships/incoming",
            get(UserRelationshipHandler::get_incoming_requests),
        )
        .route(
            "/:user_id/relationships/outgoing",
            get(UserRelationshipHandler::get_outgoing_requests),
        )
        .route(
            "/:user_id/relationships/blocked",
            get(UserRelationshipHandler::get_blocked_users),
        )
}
