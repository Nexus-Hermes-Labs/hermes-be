use axum::{
    routing::{get, patch},
    Router,
};

use crate::presentation::http::handlers::message::{
    delete_message, edit_message, get_channel_messages, get_conversation_messages,
    send_channel_message, send_conversation_message,
};
use crate::state::AppState;

pub fn channel_message_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/channels/{channel_id}/messages",
            get(get_channel_messages).post(send_channel_message),
        )
}

pub fn conversation_message_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/conversations/{conversation_id}/messages",
            get(get_conversation_messages).post(send_conversation_message),
        )
}

pub fn message_routes() -> Router<AppState> {
    Router::new()
        .route("/messages/{message_id}", patch(edit_message).delete(delete_message))
}
