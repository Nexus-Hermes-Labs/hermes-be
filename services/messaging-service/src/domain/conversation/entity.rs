use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::ConversationError;
use super::valueobject::ConversationType;

/// Conversation aggregate root (DM or Group DM).
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct Conversation {
    id: Uuid,
    conversation_type: ConversationType,
    created_at: DateTime<Utc>,
}

impl Conversation {
    #[must_use]
    pub fn new_dm() -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_type: ConversationType::Dm,
            created_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn new_group_dm() -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_type: ConversationType::GroupDm,
            created_at: Utc::now(),
        }
    }

    #[must_use]
    pub const fn from_persisted(
        id: Uuid,
        conversation_type: ConversationType,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { id, conversation_type, created_at }
    }

    #[must_use] pub const fn id(&self) -> Uuid { self.id }
    #[must_use] pub const fn conversation_type(&self) -> ConversationType { self.conversation_type }
    #[must_use] pub const fn created_at(&self) -> DateTime<Utc> { self.created_at }
    #[must_use] pub const fn is_dm(&self) -> bool { matches!(self.conversation_type, ConversationType::Dm) }
}

/// Conversation membership record.
#[derive(Debug, Clone)]
pub struct ConversationMember {
    conversation_id: Uuid,
    user_id: Uuid,
    joined_at: DateTime<Utc>,
}

impl ConversationMember {
    #[must_use]
    pub fn new(conversation_id: Uuid, user_id: Uuid) -> Self {
        Self { conversation_id, user_id, joined_at: Utc::now() }
    }

    #[must_use]
    pub const fn from_persisted(conversation_id: Uuid, user_id: Uuid, joined_at: DateTime<Utc>) -> Self {
        Self { conversation_id, user_id, joined_at }
    }

    #[must_use] pub const fn conversation_id(&self) -> Uuid { self.conversation_id }
    #[must_use] pub const fn user_id(&self) -> Uuid { self.user_id }
    #[must_use] pub const fn joined_at(&self) -> DateTime<Utc> { self.joined_at }
}

/// Validates member list before creating the conversation.
pub fn validate_members(
    conversation_type: ConversationType,
    members: &[Uuid],
) -> Result<(), ConversationError> {
    match conversation_type {
        ConversationType::Dm => {
            if members.len() != 2 {
                return Err(ConversationError::InvalidDmMemberCount);
            }
        }
        ConversationType::GroupDm => {
            if !(3..=10).contains(&members.len()) {
                return Err(ConversationError::InvalidGroupDmMemberCount);
            }
        }
    }
    Ok(())
}
