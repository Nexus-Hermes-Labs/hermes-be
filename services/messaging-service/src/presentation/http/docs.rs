#![allow(clippy::needless_for_each)]
use utoipa::OpenApi;

#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::conversation::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::conversation::response::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::message::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::message::response::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::reaction::response::*;

#[allow(clippy::wildcard_imports)]
use crate::presentation::http::handlers::{
    conversation::*, message::*, reaction::*,
};
use common::observability::health::{
    ComponentHealth, HealthChecks, HealthResponse, __path_health_handler, __path_liveness_handler,
    __path_readiness_handler,
};

/// `OpenAPI` documentation for the messaging service.
#[allow(clippy::needless_for_each)]
#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        // Messages – channel
        get_channel_messages,
        send_channel_message,
        // Messages – conversation
        get_conversation_messages,
        send_conversation_message,
        // Messages – mutation
        edit_message,
        delete_message,
        // Reactions
        get_reactions,
        count_reactions,
        add_reaction,
        remove_reaction,
        // Conversations
        get_my_conversations,
        get_conversation,
        open_dm,
        create_group_dm,
        get_conversation_members,
        add_member,
        leave_conversation,
        // Health
        health_handler,
        liveness_handler,
        readiness_handler,
    ),
    components(schemas(
        SendMessageRequest,
        EditMessageRequest,
        MessageResponse,
        MessageListResponse,
        ReactionResponse,
        ReactionCountResponse,
        OpenDmRequest,
        CreateGroupDmRequest,
        AddMemberRequest,
        ConversationResponse,
        ConversationListResponse,
        ConversationMemberResponse,
        HealthResponse,
        HealthChecks,
        ComponentHealth,
    )),
    tags(
        (name = "messages",      description = "Channel and conversation messages"),
        (name = "reactions",     description = "Message reactions"),
        (name = "conversations", description = "DMs and group DMs"),
        (name = "health",        description = "Health check endpoints"),
    ),
    info(
        title = "Messaging Service API",
        version = "0.1.0",
        description = "Handles messages, reactions, and conversations (DMs/Group DMs)"
    )
)]
pub struct ApiDoc;
