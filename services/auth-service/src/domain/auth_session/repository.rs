use super::AuthSession;
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use uuid::Uuid;

/// Repository for the `AuthSession` aggregate.
///
/// --------------------------------------------------
/// 🧩 INHERITED FROM `Repository<AuthSession, Uuid>`
/// --------------------------------------------------
///
/// The following methods are provided by the base trait
/// and MUST be implemented through `Repository`:
///
/// ### Core CRUD
/// - `find_by_id(id: Uuid)`
/// - `save(entity: &AuthSession)`
/// - `update(entity: &AuthSession)`
///
/// ### Utility
/// - `find_all()`
/// - `delete(id: ID)`
/// - `exists(id: Uuid)`
/// - `count()`
///
/// ⚠️ Do NOT redeclare these here.
/// This trait only defines domain-specific session queries.
///
/// --------------------------------------------------
/// 🎯 DOMAIN-SPECIFIC METHODS
/// --------------------------------------------------
#[async_trait]
pub trait AuthSessionRepository: Repository<AuthSession, Uuid, Error = RepositoryError> + Send + Sync {
    // ============================================
    // SESSION LOOKUPS
    // ============================================

    /// Find session by refresh token hash
    async fn find_by_refresh_token_hash(
        &self,
        hash: &str,
    ) -> Result<Option<AuthSession>, Self::Error>;

    // ============================================
    // USER-SPECIFIC QUERIES
    // ============================================

    /// Find all active (non-revoked, non-expired) sessions for a credential
    async fn find_active_by_credential_id(&self, credential_id: Uuid) -> Result<Vec<AuthSession>, Self::Error>;

    /// Find all sessions (including revoked/expired) for a credential
    async fn find_all_by_credential_id(&self, credential_id: Uuid) -> Result<Vec<AuthSession>, Self::Error>;

    // ============================================
    // BULK OPERATIONS
    // ============================================

    /// Revoke all sessions for a credential (logout from all devices)
    async fn revoke_all_by_credential_id(&self, credential_id: Uuid) -> Result<usize, Self::Error>;

    /// Delete expired sessions (cleanup job)
    async fn delete_expired_sessions(&self) -> Result<usize, Self::Error>;

    /// Delete revoked sessions older than N days (cleanup job)
    async fn delete_old_revoked_sessions(&self, days: i64) -> Result<usize, Self::Error>;
}
