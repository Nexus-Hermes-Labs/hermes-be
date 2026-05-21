use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::domain::event::IntoEventEnvelope;

use crate::domain::conversation::Conversation;
use crate::domain::message::Message;
use crate::domain::reaction::Reaction;

pub const EVENT_TYPE_MESSAGE_CREATED: &str = "messaging.message.created";
pub const EVENT_TYPE_MESSAGE_UPDATED: &str = "messaging.message.updated";
pub const EVENT_TYPE_MESSAGE_DELETED: &str = "messaging.message.deleted";
pub const EVENT_TYPE_REACTION_ADDED: &str = "messaging.reaction.added";
pub const EVENT_TYPE_REACTION_REMOVED: &str = "messaging.reaction.removed";
pub const EVENT_TYPE_CONVERSATION_CREATED: &str = "messaging.conversation.created";
pub const EVENT_TYPE_CONVERSATION_MEMBER_JOINED: &str = "messaging.conversation.member.joined";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingMessageCreatedEvent {
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl MessagingMessageCreatedEvent {
    pub fn from_message(m: &Message) -> Self {
        Self {
            message_id: m.id(),
            channel_id: m.channel_id(),
            conversation_id: m.conversation_id(),
            user_id: m.user_id(),
            content: m.content().as_str().to_string(),
            created_at: m.created_at(),
        }
    }
}

impl IntoEventEnvelope for MessagingMessageCreatedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_CREATED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingMessageUpdatedEvent {
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub content: String,
    pub edited_at: Option<DateTime<Utc>>,
}

impl MessagingMessageUpdatedEvent {
    pub fn from_message(m: &Message) -> Self {
        Self {
            message_id: m.id(),
            channel_id: m.channel_id(),
            conversation_id: m.conversation_id(),
            content: m.content().as_str().to_string(),
            edited_at: m.edited_at(),
        }
    }
}

impl IntoEventEnvelope for MessagingMessageUpdatedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_UPDATED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingMessageDeletedEvent {
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub user_id: Uuid,
}

impl MessagingMessageDeletedEvent {
    pub fn from_message(m: &Message) -> Self {
        Self {
            message_id: m.id(),
            channel_id: m.channel_id(),
            conversation_id: m.conversation_id(),
            user_id: m.user_id(),
        }
    }
}

impl IntoEventEnvelope for MessagingMessageDeletedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MESSAGE_DELETED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingReactionAddedEvent {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
}

impl MessagingReactionAddedEvent {
    pub fn from_reaction(r: &Reaction) -> Self {
        Self {
            message_id: r.message_id(),
            user_id: r.user_id(),
            emoji: r.emoji().as_str().to_string(),
        }
    }
}

impl IntoEventEnvelope for MessagingReactionAddedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_REACTION_ADDED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingReactionRemovedEvent {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
}

impl IntoEventEnvelope for MessagingReactionRemovedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_REACTION_REMOVED
    }
    fn aggregate_id(&self) -> Uuid {
        self.message_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConversationCreatedEvent {
    pub conversation_id: Uuid,
    pub conversation_type: String,
    pub member_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl MessagingConversationCreatedEvent {
    pub fn new(conv: &Conversation, member_ids: &[Uuid]) -> Self {
        Self {
            conversation_id: conv.id(),
            conversation_type: conv.conversation_type().as_str().to_string(),
            member_ids: member_ids.to_vec(),
            created_at: conv.created_at(),
        }
    }
}

impl IntoEventEnvelope for MessagingConversationCreatedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_CONVERSATION_CREATED
    }
    fn aggregate_id(&self) -> Uuid {
        self.conversation_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConversationMemberJoinedEvent {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
}

impl IntoEventEnvelope for MessagingConversationMemberJoinedEvent {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_CONVERSATION_MEMBER_JOINED
    }
    fn aggregate_id(&self) -> Uuid {
        self.conversation_id
    }
}
