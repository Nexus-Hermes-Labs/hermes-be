use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Guild;

/// Guild repository trait
#[async_trait]
pub trait GuildRepository: Repository<Guild, Uuid, Error = RepositoryError> + Send + Sync {
    // Inherits from Repository<Guild, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<Guild>>
    // - save(entity: &Guild) -> Result<()>
    // - update(entity: &Guild) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>

    // ============================================
    // OWNER QUERIES
    // ============================================

    /// Get all guilds owned by a user
    async fn find_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<Guild>, Self::Error>;

    // ============================================
    // SEARCH & DISCOVERY
    // ============================================

    /// Search public guilds by name
    async fn search_public(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Guild>, Self::Error>;

    /// Get guilds by multiple IDs (batch lookup)
    async fn find_by_ids(
        &self,
        ids: Vec<Uuid>,
    ) -> Result<Vec<Guild>, Self::Error>;

    // ============================================
    // MEMBER COUNT
    // ============================================

    /// Atomically increment member count
    async fn increment_member_count(
        &self,
        guild_id: Uuid,
    ) -> Result<(), Self::Error>;

    /// Atomically decrement member count
    async fn decrement_member_count(
        &self,
        guild_id: Uuid,
    ) -> Result<(), Self::Error>;
}
