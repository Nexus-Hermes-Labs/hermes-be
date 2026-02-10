use crate::domain::auth_credential::{AuthCredential, AuthCredentialRepository, Email};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::models::{AuthCredentialInsert, AuthCredentialRow, AuthCredentialUpdate};

/// PostgreSQL implementation of AuthCredentialRepository
pub struct PostgresAuthCredentialRepository {
    pool: PgPool,
}

impl PostgresAuthCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ============================================
// BASE REPOSITORY TRAIT IMPLEMENTATION
// ============================================

#[async_trait]
impl Repository<AuthCredential, Uuid> for PostgresAuthCredentialRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthCredential>, Self::Error> {
        debug!(user_id = %id, "Finding auth credential by ID");

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            "SELECT * FROM auth_credentials WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<AuthCredential>, Self::Error> {
        debug!("Finding all auth credentials");

        let rows = sqlx::query_as::<_, AuthCredentialRow>(
            "SELECT * FROM auth_credentials WHERE deleted_at IS NULL ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, credential: &AuthCredential) -> Result<(), Self::Error> {
        let insert: AuthCredentialInsert = credential.into();

        info!(
            user_id = %insert.id,
            email = %insert.email,
            "Saving auth credential"
        );

        sqlx::query(
            r#"
            INSERT INTO auth_credentials (id, email, password_hash)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(insert.id)
        .bind(insert.email)
        .bind(insert.password_hash)
        .execute(&self.pool)
        .await?;

        debug!(user_id = %insert.id, "Auth credential saved");
        Ok(())
    }

    async fn update(&self, credential: &AuthCredential) -> Result<(), Self::Error> {
        let update: AuthCredentialUpdate = credential.into();

        debug!(user_id = %update.id, "Updating auth credential");

        let result = sqlx::query(
            r#"
            UPDATE auth_credentials SET
                email = $2,
                password_hash = $3,
                email_verified = $4,
                email_verification_token = $5,
                email_verification_expires_at = $6,
                failed_login_attempts = $7,
                locked_until = $8,
                last_login_at = $9,
                last_login_ip = $10,
                account_status = $11,
                deleted_at = $12,
                password_reset_token = $13,
                password_reset_expires_at = $14,
                password_changed_at = $15
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(update.id)
        .bind(update.email)
        .bind(update.password_hash)
        .bind(update.email_verified)
        .bind(update.email_verification_token)
        .bind(update.email_verification_expires_at)
        .bind(update.failed_login_attempts)
        .bind(update.locked_until)
        .bind(update.last_login_at)
        .bind(update.last_login_ip)
        .bind(update.account_status)
        .bind(update.deleted_at)
        .bind(update.password_reset_token)
        .bind(update.password_reset_expires_at)
        .bind(update.password_changed_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            error!(user_id = %update.id, "No rows updated - credential not found");
            return Err(RepositoryError::NotFound(format!(
                "Credential with ID {} not found",
                update.id
            )));
        }

        debug!(user_id = %update.id, "Auth credential updated");
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        info!(user_id = %id, "Soft deleting auth credential");

        let result = sqlx::query(
            r#"
            UPDATE auth_credentials
            SET account_status = 'deleted', deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Credential with ID {} not found",
                id
            )));
        }

        debug!(user_id = %id, "Auth credential soft deleted");
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        debug!(user_id = %id, "Checking if auth credential exists");

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM auth_credentials WHERE id = $1 AND deleted_at IS NULL)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        debug!("Counting auth credentials");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM auth_credentials WHERE deleted_at IS NULL")
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }
}

// ============================================
// DOMAIN-SPECIFIC REPOSITORY TRAIT
// ============================================

#[async_trait]
impl AuthCredentialRepository for PostgresAuthCredentialRepository {
    async fn find_by_email(&self, email: &Email) -> Result<Option<AuthCredential>, Self::Error> {
        debug!(email = %email.as_str(), "Finding auth credential by email");

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            "SELECT * FROM auth_credentials WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_by_verification_token(
        &self,
        token: &str,
    ) -> Result<Option<AuthCredential>, Self::Error> {
        debug!("Finding credential by email verification token");

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            r#"
            SELECT * FROM auth_credentials
            WHERE email_verification_token = $1
              AND deleted_at IS NULL
              AND email_verification_expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_by_password_reset_token(
        &self,
        token: &str,
    ) -> Result<Option<AuthCredential>, Self::Error> {
        debug!("Finding credential by password reset token");

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            r#"
            SELECT * FROM auth_credentials
            WHERE password_reset_token = $1
              AND deleted_at IS NULL
              AND password_reset_expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, Self::Error> {
        debug!(email = %email.as_str(), "Checking if email exists");

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM auth_credentials WHERE email = $1 AND deleted_at IS NULL)",
        )
        .bind(email.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn find_all_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthCredential>, Self::Error> {
        debug!(
            limit = limit,
            offset = offset,
            "Finding paginated credentials"
        );

        let rows = sqlx::query_as::<_, AuthCredentialRow>(
            r#"
            SELECT * FROM auth_credentials
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_credential::PasswordHash;

    #[sqlx::test]
    #[ignore]
    async fn test_save_and_find_credential(pool: PgPool) {
        let repo = PostgresAuthCredentialRepository::new(pool);

        let email = Email::new("test@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let credential = AuthCredential::new(email.clone(), password_hash);

        // Save
        repo.save(&credential).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(credential.id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email().as_str(), "test@example.com");

        // Find by email
        let found = repo.find_by_email(&email).await.unwrap();
        assert!(found.is_some());
    }

    #[sqlx::test]
    #[ignore]
    async fn test_exists_methods(pool: PgPool) {
        let repo = PostgresAuthCredentialRepository::new(pool);

        let email = Email::new("exists@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let credential = AuthCredential::new(email.clone(), password_hash);

        // Should not exist initially
        assert!(!repo.exists(credential.id()).await.unwrap());
        assert!(!repo.exists_by_email(&email).await.unwrap());

        // Save
        repo.save(&credential).await.unwrap();

        // Should exist now
        assert!(repo.exists(credential.id()).await.unwrap());
        assert!(repo.exists_by_email(&email).await.unwrap());
    }

    #[sqlx::test]
    #[ignore]
    async fn test_count_and_pagination(pool: PgPool) {
        let repo = PostgresAuthCredentialRepository::new(pool);

        // Create 5 credentials
        for i in 0..5 {
            let email = Email::new(&format!("user{}@example.com", i)).unwrap();
            let password_hash = PasswordHash::from_hash("$argon2id$...");
            let credential = AuthCredential::new(email, password_hash);
            repo.save(&credential).await.unwrap();
        }

        // Count
        let count = repo.count().await.unwrap();
        assert_eq!(count, 5);

        // Paginated fetch
        let page1 = repo.find_all_paginated(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.find_all_paginated(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);
    }
}
