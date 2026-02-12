use async_trait::async_trait;
use uuid::Uuid;
use common::infrastructure::persistence::repository::Repository;

use super::entity::UserProfile;
use super::valueobject::Username;

/// User profile repository
#[async_trait]
pub trait UserProfileRepository: Repository<UserProfile, Uuid> + Send + Sync {
    // Inherits from Repository<UserProfile, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<UserProfile>>
    // - save(entity: &UserProfile) -> Result<()>
    // - update(entity: &UserProfile) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>
    
    // ============================================
    // USERNAME OPERATIONS
    // ============================================
    
    /// Find profile by username (unique lookup)
    async fn find_by_username(
        &self,
        username: &Username,
    ) -> Result<Option<UserProfile>, Self::Error>;
    
    /// Check if username is available (not taken)
    async fn is_username_available(
        &self,
        username: &Username,
    ) -> Result<bool, Self::Error>;
    
    /// Check if username exists (taken)
    async fn exists_by_username(
        &self,
        username: &Username,
    ) -> Result<bool, Self::Error>;
    
    // ============================================
    // SEARCH & LOOKUP
    // ============================================
    
    /// Search profiles by username or display name
    async fn search(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, Self::Error>;
    
    /// Get profiles by multiple IDs (batch lookup)
    async fn find_by_ids(
        &self,
        ids: Vec<Uuid>,
    ) -> Result<Vec<UserProfile>, Self::Error>;
    
    // ============================================
    // PRESENCE QUERIES
    // ============================================
    
    /// Get all online users (for presence system)
    async fn find_online_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, Self::Error>;
}
