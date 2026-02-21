use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::presentation::http::handlers::channel::{
    create_channel, delete_channel, get_channel, list_guild_channels, update_channel,
};
use crate::state::AppState;

/// Routes requiring JWT authentication (mutating operations)
pub fn authenticated_channel_routes() -> Router<AppState> {
    Router::new()
        // Guild-scoped channel creation
        .route(
            "/guilds/:guild_id/channels",
            post(create_channel),
        )
        // Channel-scoped mutations
        .route(
            "/channels/:channel_id",
            patch(update_channel).delete(delete_channel),
        )
}

/// Public routes (read-only)
pub fn public_channel_routes() -> Router<AppState> {
    Router::new()
        .route("/guilds/:guild_id/channels", get(list_guild_channels))
        .route("/channels/:channel_id", get(get_channel))
}
