use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::presentation::http::handlers::guild_invite::{
    create_invite, get_invite, list_invites, revoke_invite, use_invite,
};
use crate::state::AppState;

/// Routes that require authentication
pub fn guild_invite_routes() -> Router<AppState> {
    Router::new()
        .route("/guilds/:guild_id/invites", post(create_invite))
        .route("/guilds/:guild_id/invites", get(list_invites))
        .route("/guilds/:guild_id/invites/:code", delete(revoke_invite))
        .route("/invites/:code/use", post(use_invite))
}

/// Routes accessible without authentication
pub fn public_invite_routes() -> Router<AppState> {
    Router::new().route("/invites/:code", get(get_invite))
}
