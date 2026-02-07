use crate::domain::user::User;
use async_trait::async_trait;
use common::pagination::{Paginated, PaginationParams};
use common::Repository;
use uuid::Uuid;
use common::persistance::error::RepositoryError;

/// User-specific repository trait for User Service domain
#[async_trait]
pub trait UserRepository: Repository<User, Uuid> + Send + Sync {
    // ─── Single lookups ──────────────────────────────

    /// Find active user by username.
    /// Used when sending a friend request by username.
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;

    // ─── Bulk lookups ────────────────────────────────

    /// Fetch multiple users by IDs in one query.
    /// Used by Domain Service for cross-aggregate enrichment
    /// (e.g. attach UserSnapshot to each friend-request row).
    /// Silently skips IDs that don't match an active user.
    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, Self::Error>;

    // ─── Search ──────────────────────────────────────

    /// Full-text search across username, display_name, and bio.
    /// Backed by the GIN tsvector index on the users table.
    /// Returns paginated results sorted by relevance.
    async fn search(
        &self,
        query: &str,
        params: &PaginationParams,
    ) -> Result<Paginated<User>, Self::Error>;
}



#[async_trait]
pub trait DiscriminatorRepository: Send + Sync {
    /// Get the highest discriminator for a username
    async fn find_max_discriminator(
        &self,
        username: &str,
    ) -> Result<Option<String>, RepositoryError>;

    /// Check if username#discriminator combination exists
    async fn exists(
        &self,
        username: &str,
        discriminator: &str,
    ) -> Result<bool, RepositoryError>;

    /// Count total users with same username
    async fn count_by_username(
        &self,
        username: &str,
    ) -> Result<i64, RepositoryError>;
}