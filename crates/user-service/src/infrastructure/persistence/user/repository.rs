use super::entity::UserEntity;

use crate::domain::user::filters::UserFilters;
use crate::domain::user::UserRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use common::persistance::error::RepositoryError;
use common::Repository;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Implement generic Repository trait
#[async_trait]
impl Repository<UserEntity, Uuid> for PostgresUserRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserEntity>, Self::Error> {
        let user = sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_all(&self) -> Result<Vec<UserEntity>, Self::Error> {
        let users = sqlx::query_as::<_, UserEntity>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    async fn save(&self, entity: &UserEntity) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, username, password_hash, first_name, last_name, role, is_active, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(entity.id)
        .bind(&entity.email)
        .bind(&entity.username)
        .bind(&entity.password_hash)
        .bind(&entity.first_name)
        .bind(&entity.last_name)
        .bind(&entity.role)
        .bind(entity.is_active)
        .bind(entity.email_verified)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(
                        "User with this email or username already exists".to_string(),
                    );
                }
            }
            RepositoryError::Database(e)
        })?;
        Ok(())
    }

    async fn update(&self, entity: &UserEntity) -> Result<(), Self::Error> {
        let result = sqlx::query(
            r#"
            UPDATE users SET
                email = $2,
                username = $3,
                password_hash = $4,
                first_name = $5,
                last_name = $6,
                role = $7,
                is_active = $8,
                email_verified = $9,
                updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(entity.id)
        .bind(&entity.email)
        .bind(&entity.username)
        .bind(&entity.password_hash)
        .bind(&entity.first_name)
        .bind(&entity.last_name)
        .bind(&entity.role)
        .bind(entity.is_active)
        .bind(entity.email_verified)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", entity.id));
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", id));
        }
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}

// Implement UserRepository trait (user-specific methods)
#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserEntity>, RepositoryError> {
        let user = sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, RepositoryError> {
        let user = sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_all_paginated(
        &self,
        params: PaginationParams,
    ) -> Result<Paginated<UserEntity>, RepositoryError> {
        let total = self.count().await?;

        let users = sqlx::query_as::<_, UserEntity>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(&self.pool)
        .await?;

        Ok(Paginated::new(users, total, params))
    }

    async fn list(
        &self,
        filters: &UserFilters,
        params: &PaginationParams,
    ) -> Result<Paginated<UserEntity>, RepositoryError> {
        // Build WHERE conditions
        let mut conditions = vec!["1=1".to_string()];

        // Active/deleted filter
        match filters.is_active {
            Some(true) => conditions.push("deleted_at IS NULL".to_string()),
            Some(false) => conditions.push("deleted_at IS NOT NULL".to_string()),
            None => {} // All users
        }

        if let Some(ref email) = filters.email {
            conditions.push(format!("email ILIKE '%{}%'", email));
        }

        if let Some(ref username) = filters.username {
            conditions.push(format!("username ILIKE '%{}%'", username));
        }

        if let Some(ref role) = filters.role {
            conditions.push(format!("role = '{:?}'", role).to_lowercase());
        }

        if let Some(ref search) = filters.search {
            conditions.push(format!(
                "(email ILIKE '%{0}%' OR username ILIKE '%{0}%' OR first_name ILIKE '%{0}%' OR last_name ILIKE '%{0}%')",
                search
            ));
        }

        let where_clause = conditions.join(" AND ");

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);
        let total: (i64,) = sqlx::query_as(&count_query).fetch_one(&self.pool).await?;

        // If no results, return empty
        if total.0 == 0 {
            return Ok(Paginated::empty(params.clone()));
        }

        // Data query
        let data_query = format!(
            "SELECT * FROM users WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause,
            params.limit(),
            params.offset()
        );

        let items = sqlx::query_as::<_, UserEntity>(&data_query)
            .fetch_all(&self.pool)
            .await?;

        Ok(Paginated::new(items, total.0, params.clone()))
    }
}
