use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::presentation::http::handlers::guild::*;
use crate::state::AppState;

pub fn guild_routes() -> Router<AppState> {
    Router::new()
        .route("/guilds", post(create_guild))
        .route("/guilds/search", get(search_guilds))
        .route("/guilds/:guild_id", get(get_guild))
        .route("/guilds/:guild_id", patch(update_guild))
        .route("/guilds/:guild_id", delete(delete_guild))
}
