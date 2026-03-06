use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Guild;

/// Guild repository trait
#[async_trait]
pub trait GuildRepository: Repository<Guild, Uuid, Error = RepositoryError> + Send + Sync {
    // ============================================
    // OWNER QUERIES
    // ============================================

    /// Get all guilds owned by a user
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Guild>, Self::Error>;

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
    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Guild>, Self::Error>;
}
