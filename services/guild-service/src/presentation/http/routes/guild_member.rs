use axum::{
    routing::{delete, get, put},
    Router,
};

use crate::presentation::http::handlers::guild_member::{
    assign_role, get_member, kick_member, leave_guild, list_members, remove_role,
};
use crate::state::AppState;

pub fn guild_member_routes() -> Router<AppState> {
    Router::new()
        .route("/guilds/:guild_id/members", get(list_members))
        .route("/guilds/:guild_id/members/@me", delete(leave_guild))
        .route("/guilds/:guild_id/members/:user_id", get(get_member))
        .route("/guilds/:guild_id/members/:user_id", delete(kick_member))
        .route("/guilds/:guild_id/members/:user_id/roles", put(assign_role))
        .route(
            "/guilds/:guild_id/members/:user_id/roles/:role_id",
            delete(remove_role),
        )
}
