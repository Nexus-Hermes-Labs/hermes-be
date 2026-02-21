use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Channel;

/// Channel repository trait
#[async_trait]
pub trait ChannelRepository:
    Repository<Channel, Uuid, Error = RepositoryError> + Send + Sync
{
    // Inherits from Repository<Channel, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<Channel>>
    // - find_all() -> Result<Vec<Channel>>
    // - save(entity: &Channel) -> Result<()>
    // - update(entity: &Channel) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>

    // ============================================
    // GUILD QUERIES
    // ============================================

    /// Get all active channels in a guild, ordered by position ascending.
    async fn find_by_guild(&self, guild_id: Uuid) -> Result<Vec<Channel>, Self::Error>;

    /// Get the maximum position value for channels in a guild.
    async fn max_position(&self, guild_id: Uuid) -> Result<i32, Self::Error>;
}
