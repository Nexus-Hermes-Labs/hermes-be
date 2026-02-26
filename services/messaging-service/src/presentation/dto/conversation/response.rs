use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{Conversation, ConversationMember};

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub conversation_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<Conversation> for ConversationResponse {
    fn from(c: Conversation) -> Self {
        Self {
            id: c.id(),
            conversation_type: c.conversation_type().as_str().to_string(),
            created_at: c.created_at(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationMemberResponse {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

impl From<ConversationMember> for ConversationMemberResponse {
    fn from(m: ConversationMember) -> Self {
        Self {
            conversation_id: m.conversation_id(),
            user_id: m.user_id(),
            joined_at: m.joined_at(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationResponse>,
    pub total: usize,
}
