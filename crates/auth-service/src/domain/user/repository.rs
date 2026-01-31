use crate::infrastructure::persistence::user_repository::entity::AuthUserEntity;
use async_trait::async_trait;
use common::persistance::error::RepositoryError;
use common::Repository;
use uuid::Uuid;

/// User-specific repository trait extending generic Repository
#[async_trait]
pub trait AuthUserRepository: Repository<AuthUserEntity, Uuid> + Send + Sync {
    /// Find user by email - for login
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthUserEntity>, RepositoryError>;

    /// Find user by username - for uniqueness check
    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AuthUserEntity>, RepositoryError>;

    /// Check if email exists
    async fn exists_by_email(&self, email: &str) -> Result<bool, RepositoryError>;

    /// Check if username exists
    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError>;
}
