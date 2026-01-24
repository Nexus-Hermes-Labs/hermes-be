use crate::domain::user::repository::AuthUserRepository;
use crate::infrastructure::persistence::user::entity::AuthUserEntity;
use async_trait::async_trait;
use common::persistance::error::RepositoryError;
use common::Repository;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresAuthUserRepository {
    pool: PgPool,
}

impl PostgresAuthUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ============================================
// Generic Repository Implementation
// ============================================

#[async_trait]
impl Repository<AuthUserEntity, Uuid> for PostgresAuthUserRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthUserEntity>, Self::Error> {
        let user = sqlx::query_as!(
            AuthUserEntity,
            r#"
            SELECT
                id,
                username,
                email,
                password_hash,
                email_verified,
                email_verification_token,
                role,
                is_active,
                created_at,
                updated_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(user)
    }

    async fn find_all(&self) -> Result<Vec<AuthUserEntity>, Self::Error> {
        let users = sqlx::query_as!(
            AuthUserEntity,
            r#"
            SELECT
                id,
                username,
                email,
                password_hash,
                email_verified,
                email_verification_token,
                role,
                is_active,
                created_at,
                updated_at
            FROM users
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(users)
    }

    async fn save(&self, entity: &AuthUserEntity) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (
                id,
                username,
                email,
                password_hash,
                email_verified,
                email_verification_token,
                role,
                is_active,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            entity.id,
            entity.username,
            entity.email,
            entity.password_hash,
            entity.email_verified,
            entity.email_verification_token,
            entity.role,
            entity.is_active,
            entity.created_at,
            entity.updated_at
        )
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

    async fn update(&self, entity: &AuthUserEntity) -> Result<(), Self::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE users SET
                username = $2,
                email = $3,
                password_hash = $4,
                email_verified = $5,
                email_verification_token = $6,
                role = $7,
                is_active = $8,
                updated_at = $9
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            entity.id,
            entity.username,
            entity.email,
            entity.password_hash,
            entity.email_verified,
            entity.email_verification_token,
            entity.role,
            entity.is_active,
            entity.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", entity.id));
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        // Soft delete
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", id));
        }

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE id = $1 AND deleted_at IS NULL
            ) as "exists!"
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM users
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(count)
    }
}

// ============================================
// Domain-Specific Repository Implementation
// ============================================

#[async_trait]
impl AuthUserRepository for PostgresAuthUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthUserEntity>, RepositoryError> {
        let user = sqlx::query_as!(
            AuthUserEntity,
            r#"
            SELECT
                id,
                username,
                email,
                password_hash,
                email_verified,
                email_verification_token,
                role,
                is_active,
                created_at,
                updated_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(user)
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AuthUserEntity>, RepositoryError> {
        let user = sqlx::query_as!(
            AuthUserEntity,
            r#"
            SELECT
                id,
                username,
                email,
                password_hash,
                email_verified,
                email_verification_token,
                role,
                is_active,
                created_at,
                updated_at
            FROM users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(user)
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE email = $1 AND deleted_at IS NULL
            ) as "exists!"
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE username = $1 AND deleted_at IS NULL
            ) as "exists!"
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }
}
