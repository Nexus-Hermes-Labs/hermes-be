use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::conversation::{
    Conversation, ConversationError, ConversationMember, ConversationType,
};

#[derive(Debug, Clone, FromRow)]
pub struct ConversationRow {
    pub id:              Uuid,
    pub r#type:          String,
    pub created_at:      DateTime<Utc>,
}

impl TryFrom<ConversationRow> for Conversation {
    type Error = ConversationError;

    fn try_from(row: ConversationRow) -> Result<Self, Self::Error> {
        let conv_type = row.r#type.parse::<ConversationType>()?;
        Ok(Self::from_persisted(row.id, conv_type, row.created_at))
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ConversationMemberRow {
    pub conversation_id: Uuid,
    pub user_id:         Uuid,
    pub joined_at:       DateTime<Utc>,
}

impl From<ConversationMemberRow> for ConversationMember {
    fn from(row: ConversationMemberRow) -> Self {
        Self::from_persisted(row.conversation_id, row.user_id, row.joined_at)
    }
}

impl From<&ConversationMember> for ConversationMemberRow {
    fn from(m: &ConversationMember) -> Self {
        Self {
            conversation_id: m.conversation_id(),
            user_id:         m.user_id(),
            joined_at:       m.joined_at(),
        }
    }
}
