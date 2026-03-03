use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::message::Message;

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
    pub edited_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(m: Message) -> Self {
        Self {
            id: m.id(),
            channel_id: m.channel_id(),
            user_id: m.user_id(),
            content: m.content().as_str().to_string(),
            message_type: m.message_type().as_str().to_string(),
            reply_to_id: m.reply_to_id(),
            edited_at: m.edited_at(),
            is_deleted: m.is_deleted(),
            created_at: m.created_at(),
            updated_at: m.updated_at(),
        }
    }
}

impl From<&Message> for MessageResponse {
    fn from(m: &Message) -> Self {
        Self {
            id: m.id(),
            channel_id: m.channel_id(),
            user_id: m.user_id(),
            content: m.content().as_str().to_string(),
            message_type: m.message_type().as_str().to_string(),
            reply_to_id: m.reply_to_id(),
            edited_at: m.edited_at(),
            is_deleted: m.is_deleted(),
            created_at: m.created_at(),
            updated_at: m.updated_at(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
}
