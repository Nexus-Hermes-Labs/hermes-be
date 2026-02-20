use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::presentation::http::handlers::guild_role::{
    create_role, delete_role, get_role, list_roles, update_role,
};
use crate::state::AppState;

pub fn guild_role_routes() -> Router<AppState> {
    Router::new()
        .route("/guilds/:guild_id/roles", post(create_role))
        .route("/guilds/:guild_id/roles", get(list_roles))
        .route("/guilds/:guild_id/roles/:role_id", get(get_role))
        .route("/guilds/:guild_id/roles/:role_id", patch(update_role))
        .route("/guilds/:guild_id/roles/:role_id", delete(delete_role))
}
