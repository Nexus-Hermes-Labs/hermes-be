use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::message::Message;
use crate::domain::reaction::Reaction;

/// Transactional writer for channel messages. Used inside the chat `UoW` so
/// that a message INSERT and its outbox event are committed atomically.
#[async_trait]
pub trait MessageWriter: Send + Sync {
    async fn save(&self, message: &Message) -> Result<(), RepositoryError>;
    async fn update(&self, message: &Message) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait ReactionWriter: Send + Sync {
    async fn save(&self, reaction: &Reaction) -> Result<(), RepositoryError>;
    async fn delete_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<u64, RepositoryError>;
}
