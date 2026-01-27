use crate::domain::user::filters::UserFilters;
use crate::infrastructure::persistence::user::entity::UserEntity;
use async_trait::async_trait;
use common::pagination::{Paginated, PaginationParams};
use common::persistance::error::RepositoryError;
use common::Repository;
use uuid::Uuid;

/// User-specific repository trait extending generic Repository
#[async_trait]
pub trait UserRepository: Repository<UserEntity, Uuid> + Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserEntity>, RepositoryError>;
    async fn find_by_username(&self, username: &str)
        -> Result<Option<UserEntity>, RepositoryError>;
    async fn find_all_paginated(
        &self,
        params: PaginationParams,
    ) -> Result<Paginated<UserEntity>, RepositoryError>;
    async fn list(
        &self,
        filters: &UserFilters,
        params: &PaginationParams,
    ) -> Result<Paginated<UserEntity>, RepositoryError>;
}
