use crate::presentation::http::handlers::user_me;
use crate::state::AppState;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

/// Main router for `@me` endpoints.
/// These routes handle operations for the currently authenticated user.
pub fn user_me_routes() -> Router<AppState> {
    Router::new().nest(
        "/@me",
        Router::new()
            .merge(profile_routes())
            .merge(privacy_routes())
            .merge(relationship_routes()),
    )
}

// --- Sub-Routers for better organization ---

/// Profile Management
/// Endpoints related to the user's personal identity and status.
fn profile_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(user_me::get_profile)) // Get current user details
        .route("/", patch(user_me::update_profile)) // Partial update of profile
        .route("/", delete(user_me::delete_profile)) // Account deactivation/deletion
        .route("/username", put(user_me::change_username)) // Change display/unique name
        .route("/status", put(user_me::update_status)) // Online, Idle, DND, Offline
        .route("/custom-status", put(user_me::set_custom_status))
        .route("/custom-status", delete(user_me::clear_custom_status))
}

/// Privacy & Security Settings
/// Endpoints for managing visibility, DM permissions, and safety presets.
fn privacy_routes() -> Router<AppState> {
    Router::new().nest(
        "/privacy",
        Router::new()
            .route("/", get(user_me::get_privacy_settings))
            .route("/dm", put(user_me::update_dm_privacy))
            .route(
                "/friend-requests",
                put(user_me::update_friend_request_privacy),
            )
            .route("/visibility", patch(user_me::update_visibility))
            .route("/content", patch(user_me::update_content_settings))
            .route("/preset", post(user_me::apply_preset)), // Apply "Private" or "Public" presets
    )
}

/// Social & Relationships
/// Endpoints for friends, friend requests, and blocking.
fn relationship_routes() -> Router<AppState> {
    Router::new().nest(
        "/relationships",
        Router::new()
            // Action-based routes (Requests & Blocking)
            .route("/request", post(user_me::send_friend_request))
            .route("/request/accept", put(user_me::accept_friend_request))
            .route("/request/decline", put(user_me::decline_friend_request))
            .route("/block", post(user_me::block_user))
            // List-based routes
            .route("/friends", get(user_me::get_friends))
            .route("/incoming", get(user_me::get_incoming_requests))
            .route("/outgoing", get(user_me::get_outgoing_requests))
            .route("/blocked", get(user_me::get_blocked_users))
            // Target-specific routes
            .route("/friend/:target_user_id", delete(user_me::remove_friend))
            .route("/block/:target_user_id", delete(user_me::unblock_user))
            .route("/:target_user_id", get(user_me::get_relationship)),
    )
}
