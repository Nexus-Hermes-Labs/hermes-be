use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::{Conversation, ConversationMember};

#[async_trait]
pub trait ConversationRepository:
    Repository<Conversation, Uuid, Error = RepositoryError> + Send + Sync
{
    /// Save a batch of conversation members (single-aggregate, non-transactional).
    ///
    /// Used when adding a member to an existing conversation. For atomic
    /// creation of a new conversation with its initial members, use
    /// [`MessagingUnitOfWork`](crate::application::ports::unit_of_work::MessagingUnitOfWork).
    async fn save_members(&self, members: &[ConversationMember]) -> Result<(), Self::Error>;

    /// All conversations a user is part of (for listing a user's DMs).
    async fn find_by_member(&self, user_id: Uuid) -> Result<Vec<Conversation>, Self::Error>;

    /// All members in a conversation.
    async fn find_members(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<ConversationMember>, Self::Error>;

    /// Find the existing DM between exactly two users (returns None if it doesn't exist).
    async fn find_dm_between(
        &self,
        user_id_a: Uuid,
        user_id_b: Uuid,
    ) -> Result<Option<Conversation>, Self::Error>;

    /// Check if a user is a member of a conversation.
    async fn is_member(&self, conversation_id: Uuid, user_id: Uuid) -> Result<bool, Self::Error>;

    /// Remove a user from a group DM.
    async fn remove_member(&self, conversation_id: Uuid, user_id: Uuid) -> Result<(), Self::Error>;
}
