use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::presentation::http::handlers::conversation::{
    add_member, create_group_dm, get_conversation, get_conversation_members,
    get_my_conversations, leave_conversation, open_dm,
};
use crate::state::AppState;

pub fn conversation_routes() -> Router<AppState> {
    Router::new()
        .route("/conversations/@me", get(get_my_conversations))
        .route("/conversations/{conversation_id}", get(get_conversation))
        .route("/conversations/dm", post(open_dm))
        .route("/conversations/group", post(create_group_dm))
        .route(
            "/conversations/{conversation_id}/members",
            get(get_conversation_members).post(add_member),
        )
        .route(
            "/conversations/{conversation_id}/members/@me",
            delete(leave_conversation),
        )
}
