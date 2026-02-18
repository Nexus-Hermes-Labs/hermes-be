use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use crate::presentation::http::handlers::UserMeHandler;
use crate::state::AppState;

/// Main router for `@me` endpoints.
/// These routes handle operations for the currently authenticated user.
pub fn user_me_routes() -> Router<AppState> {
    Router::new()
        .nest("/@me", Router::new()
            .merge(profile_routes())
            .merge(privacy_routes())
            .merge(relationship_routes())
        )
}

// --- Sub-Routers for better organization ---

/// Profile Management
/// Endpoints related to the user's personal identity and status.
fn profile_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(UserMeHandler::get_profile))                // Get current user details
        .route("/", patch(UserMeHandler::update_profile))           // Partial update of profile
        .route("/", delete(UserMeHandler::delete_profile))         // Account deactivation/deletion
        .route("/username", put(UserMeHandler::change_username))    // Change display/unique name
        .route("/status", put(UserMeHandler::update_status))        // Online, Idle, DND, Offline
        .route("/custom-status", put(UserMeHandler::set_custom_status))
        .route("/custom-status", delete(UserMeHandler::clear_custom_status))
}

/// Privacy & Security Settings
/// Endpoints for managing visibility, DM permissions, and safety presets.
fn privacy_routes() -> Router<AppState> {
    Router::new()
        .nest("/privacy", Router::new()
            .route("/", get(UserMeHandler::get_privacy_settings))
            .route("/dm", put(UserMeHandler::update_dm_privacy))
            .route("/friend-requests", put(UserMeHandler::update_friend_request_privacy))
            .route("/visibility", patch(UserMeHandler::update_visibility))
            .route("/content", patch(UserMeHandler::update_content_settings))
            .route("/preset", post(UserMeHandler::apply_preset))     // Apply "Private" or "Public" presets
        )
}

/// Social & Relationships
/// Endpoints for friends, friend requests, and blocking.
fn relationship_routes() -> Router<AppState> {
    Router::new()
        .nest("/relationships", Router::new()
            // Action-based routes (Requests & Blocking)
            .route("/request", post(UserMeHandler::send_friend_request))
            .route("/request/accept", put(UserMeHandler::accept_friend_request))
            .route("/request/decline", put(UserMeHandler::decline_friend_request))
            .route("/block", post(UserMeHandler::block_user))

            // List-based routes
            .route("/friends", get(UserMeHandler::get_friends))
            .route("/incoming", get(UserMeHandler::get_incoming_requests))
            .route("/outgoing", get(UserMeHandler::get_outgoing_requests))
            .route("/blocked", get(UserMeHandler::get_blocked_users))

            // Target-specific routes
            .route("/friend/:target_user_id", delete(UserMeHandler::remove_friend))
            .route("/block/:target_user_id", delete(UserMeHandler::unblock_user))
            .route("/:target_user_id", get(UserMeHandler::get_relationship))
        )
}