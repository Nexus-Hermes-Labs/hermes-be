use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::conversation::{Conversation, ConversationMember};

/// Transactional writer for the `conversations` table.
#[async_trait]
pub trait ConversationWriter: Send + Sync {
    async fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError>;
}

/// Transactional writer for the `conversation_members` table.
#[async_trait]
pub trait ConversationMemberWriter: Send + Sync {
    async fn save_batch(&self, members: &[ConversationMember]) -> Result<(), RepositoryError>;
}

