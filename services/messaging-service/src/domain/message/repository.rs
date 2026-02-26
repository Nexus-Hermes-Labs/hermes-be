use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Message;

#[async_trait]
pub trait MessageRepository:
    Repository<Message, Uuid, Error = RepositoryError> + Send + Sync
{
    /// Cursor-paginated messages for a guild channel (newest first).
    async fn find_by_channel(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, Self::Error>;

    /// Cursor-paginated messages for a conversation (newest first).
    async fn find_by_conversation(
        &self,
        conversation_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, Self::Error>;

    /// Count non-deleted messages in a channel.
    async fn count_by_channel(&self, channel_id: Uuid) -> Result<i64, Self::Error>;

    /// Bulk soft-delete all messages in a channel (on channel deletion).
    async fn delete_all_by_channel(&self, channel_id: Uuid) -> Result<(), Self::Error>;
}
