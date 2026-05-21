use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::domain::event::IntoEventEnvelope;

use crate::domain::message::Message;
use crate::domain::reaction::Reaction;

pub const AGGREGATE_TYPE_MESSAGE: &str = "chat_message";
pub const AGGREGATE_TYPE_REACTION: &str = "chat_reaction";

pub const EVENT_TYPE_MESSAGE_CREATED: &str = "chat.message.created";
pub const EVENT_TYPE_MESSAGE_UPDATED: &str = "chat.message.updated";
pub const EVENT_TYPE_MESSAGE_DELETED: &str = "chat.message.deleted";
pub const EVENT_TYPE_REACTION_ADDED: &str = "chat.reaction.added";
pub const EVENT_TYPE_REACTION_REMOVED: &str = "chat.reaction.removed";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageCreatedEvent {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl ChatMessageCreatedEvent {
    pub fn from_message(message: &Message) -> Self {
        Self {
            message_id: message.id(),
            channel_id: message.channel_id(),
            user_id: message.user_id(),
            content: message.content().as_str().to_string(),
            created_at: message.created_at(),
        }
    }
}

impl IntoEventEnvelope for ChatMessageCreatedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_CREATED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageUpdatedEvent {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub content: String,
    pub edited_at: Option<DateTime<Utc>>,
}

impl ChatMessageUpdatedEvent {
    pub fn from_message(message: &Message) -> Self {
        Self {
            message_id: message.id(),
            channel_id: message.channel_id(),
            content: message.content().as_str().to_string(),
            edited_at: message.edited_at(),
        }
    }
}

impl IntoEventEnvelope for ChatMessageUpdatedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_UPDATED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDeletedEvent {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
}

impl ChatMessageDeletedEvent {
    pub fn from_message(message: &Message) -> Self {
        Self {
            message_id: message.id(),
            channel_id: message.channel_id(),
            user_id: message.user_id(),
        }
    }
}

impl IntoEventEnvelope for ChatMessageDeletedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_DELETED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatReactionAddedEvent {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
}

impl ChatReactionAddedEvent {
    pub fn from_reaction(reaction: &Reaction) -> Self {
        Self {
            message_id: reaction.message_id(),
            user_id: reaction.user_id(),
            emoji: reaction.emoji().as_str().to_string(),
        }
    }
}

impl IntoEventEnvelope for ChatReactionAddedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_REACTION_ADDED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatReactionRemovedEvent {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
}

impl IntoEventEnvelope for ChatReactionRemovedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_REACTION_REMOVED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}
