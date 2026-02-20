use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::GuildRole;

/// Guild role repository trait
#[async_trait]
pub trait GuildRoleRepository:
    Repository<GuildRole, Uuid, Error = RepositoryError> + Send + Sync
{
    /// Get all roles for a guild (ordered by position desc)
    async fn find_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildRole>, Self::Error>;

    /// Get roles by multiple IDs
    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<GuildRole>, Self::Error>;

    /// Get the @everyone role for a guild
    async fn find_default_role(&self, guild_id: Uuid) -> Result<Option<GuildRole>, Self::Error>;

    /// Check if a role name already exists in the guild
    async fn exists_by_name(&self, guild_id: Uuid, name: &str) -> Result<bool, Self::Error>;
}
