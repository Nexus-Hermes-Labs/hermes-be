use async_trait::async_trait;
use common::Repository;
use uuid::Uuid;
use common::persistance::error::RepositoryError;
use crate::domain::user::User;

/// User-specific repository trait extending generic Repository
#[async_trait]
pub trait AuthUserRepository: Repository<User, Uuid> + Send + Sync {
    /// Find user by email - for login
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Self::Error>;

    /// Find user by username - for uniqueness check
    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, Self::Error>;

    /// Check if email exists
    async fn exists_by_email(&self, email: &str) -> Result<bool, Self::Error>;

    /// Check if username exists
    async fn exists_by_username(&self, username: &str) -> Result<bool, Self::Error>;
}
