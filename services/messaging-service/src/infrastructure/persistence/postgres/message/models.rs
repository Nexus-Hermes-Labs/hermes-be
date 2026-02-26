use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::message::{Message, MessageContent, MessageError, MessageTarget, MessageType};

#[derive(Debug, Clone, FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub user_id: Uuid,
    pub content: String,
    pub r#type: String,
    pub reply_to_id: Option<Uuid>,
    pub is_deleted: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = MessageError;

    fn try_from(row: MessageRow) -> Result<Self, Self::Error> {
        let content = MessageContent::new(&row.content)?;
        let message_type = row.r#type.parse::<MessageType>()?;

        let target = match (row.channel_id, row.conversation_id) {
            (Some(cid), None) => MessageTarget::Channel(cid),
            (None, Some(cvid)) => MessageTarget::Conversation(cvid),
            _ => return Err(MessageError::NotFound), // violates DB constraint
        };

        Ok(Self::from_persisted(
            row.id,
            target,
            row.user_id,
            content,
            message_type,
            row.reply_to_id,
            row.edited_at,
            row.is_deleted,
            row.created_at,
            row.updated_at,
        ))
    }
}

impl From<&Message> for MessageRow {
    fn from(m: &Message) -> Self {
        Self {
            id: m.id(),
            channel_id: m.channel_id(),
            conversation_id: m.conversation_id(),
            user_id: m.user_id(),
            content: m.content().as_str().to_string(),
            r#type: m.message_type().as_str().to_string(),
            reply_to_id: m.reply_to_id(),
            is_deleted: m.is_deleted(),
            edited_at: m.edited_at(),
            created_at: m.created_at(),
            updated_at: m.updated_at(),
        }
    }
}
