use crate::infrastructure::persistence::user::entity::UserEntity;
use async_trait::async_trait;
use common::pagination::{Paginated, PaginationParams};
use common::persistance::error::RepositoryError;
use uuid::Uuid;

/// User-specific repository trait for User Service domain
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserEntity>, RepositoryError>;

    async fn find_by_username(&self, username: &str)
        -> Result<Option<UserEntity>, RepositoryError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<UserEntity>, RepositoryError>;

    /// Bulk query for efficiency
    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserEntity>, RepositoryError>;

    async fn update(&self, user: &UserEntity) -> Result<(), RepositoryError>;

    async fn search(
        &self,
        query: &str,
        params: &PaginationParams,
    ) -> Result<Paginated<UserEntity>, RepositoryError>;
}

