#![allow(clippy::doc_markdown)]
use axum::{
    routing::{get, patch},
    Router,
};

use crate::presentation::http::handlers::message::{
    delete_message, edit_message, get_messages, send_message,
};
use crate::state::AppState;

/// Routes scoped to a channel: GET/POST /channels/{channel_id}/messages
pub fn channel_message_routes() -> Router<AppState> {
    Router::new().route(
        "/channels/:channel_id/messages",
        get(get_messages).post(send_message),
    )
}

/// Routes for individual messages: PATCH/DELETE /messages/{message_id}
pub fn message_routes() -> Router<AppState> {
    Router::new().route(
        "/messages/:message_id",
        patch(edit_message).delete(delete_message),
    )
}
