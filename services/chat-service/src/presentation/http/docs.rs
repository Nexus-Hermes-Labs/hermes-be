#![allow(clippy::needless_for_each)]
use utoipa::OpenApi;

use crate::presentation::dto::message::request::{CreateMessageRequest, EditMessageRequest};
use crate::presentation::dto::message::response::{MessageListResponse, MessageResponse};
use crate::presentation::dto::reaction::response::{ReactionCountResponse, ReactionResponse};
use crate::presentation::http::handlers::{message, reaction};

#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        message::get_messages,
        message::send_message,
        message::edit_message,
        message::delete_message,
        reaction::add_reaction,
        reaction::remove_reaction,
        reaction::get_reactions,
    ),
    components(schemas(
        CreateMessageRequest,
        EditMessageRequest,
        MessageResponse,
        MessageListResponse,
        ReactionResponse,
        ReactionCountResponse,
    )),
    tags(
        (name = "messages", description = "Channel message management"),
        (name = "reactions", description = "Message reaction management"),
    ),
    info(
        title = "Chat Service API",
        version = "1.0.0",
        description = "Hermes chat-service — channel messages and reactions"
    )
)]
pub struct ApiDoc;
