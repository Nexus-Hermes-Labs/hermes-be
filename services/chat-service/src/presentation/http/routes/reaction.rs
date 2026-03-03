use axum::{
    routing::{get, put},
    Router,
};

use crate::presentation::http::handlers::reaction::{add_reaction, get_reactions, remove_reaction};
use crate::state::AppState;

pub fn reaction_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/messages/:message_id/reactions/:emoji",
            put(add_reaction).delete(remove_reaction),
        )
        .route("/messages/:message_id/reactions", get(get_reactions))
}
