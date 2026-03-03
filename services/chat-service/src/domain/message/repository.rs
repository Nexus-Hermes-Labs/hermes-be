use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Message;

/// Channel message repository trait.
#[async_trait]
pub trait MessageRepository:
    Repository<Message, Uuid, Error = RepositoryError> + Send + Sync
{
    // Inherits from Repository<Message, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<Message>>
    // - save(entity: &Message) -> Result<()>
    // - update(entity: &Message) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>

    /// Get messages in a channel with cursor-based pagination (newest-first).
    async fn find_by_channel(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, Self::Error>;

    /// Get a single non-deleted message by ID and channel (ownership check).
    async fn find_by_id_and_channel(
        &self,
        id: Uuid,
        channel_id: Uuid,
    ) -> Result<Option<Message>, Self::Error>;

    /// Count non-deleted messages in a channel.
    async fn count_by_channel(&self, channel_id: Uuid) -> Result<i64, Self::Error>;

    /// Soft-delete all messages in a channel (used when channel is deleted).
    async fn delete_all_by_channel(&self, channel_id: Uuid) -> Result<(), Self::Error>;
}
