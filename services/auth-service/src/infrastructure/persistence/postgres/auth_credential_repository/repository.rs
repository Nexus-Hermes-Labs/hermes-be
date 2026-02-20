use crate::domain::auth_credential::{AuthCredential, AuthCredentialRepository, Email};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::models::{AuthCredentialInsert, AuthCredentialRow, AuthCredentialUpdate};

/// Column list for SELECT queries on auth_credentials.
/// Casts PG enum and INET columns to TEXT so they can be decoded into String fields.
const AUTH_CREDENTIAL_COLUMNS: &str = r#"
    id, user_id, email, password_hash,
    email_verified, email_verification_token, email_verification_expires_at,
    failed_login_attempts, locked_until, last_login_at,
    last_login_ip::TEXT as last_login_ip,
    account_status::TEXT as account_status,
    deleted_at,
    password_reset_token, password_reset_expires_at, password_changed_at,
    created_at, updated_at
"#;

/// PostgreSQL implementation of AuthCredentialRepository
#[derive(Clone)]
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

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE id = $1 AND deleted_at IS NULL",
            AUTH_CREDENTIAL_COLUMNS
        );
        let row = sqlx::query_as::<_, AuthCredentialRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<AuthCredential>, Self::Error> {
        debug!("Finding all auth credentials");

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE deleted_at IS NULL ORDER BY created_at DESC",
            AUTH_CREDENTIAL_COLUMNS
        );
        let rows = sqlx::query_as::<_, AuthCredentialRow>(&sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, credential: &AuthCredential) -> Result<(), Self::Error> {
        let insert: AuthCredentialInsert = credential.into();

        info!(
            credential_id = %insert.id,
            user_id = %insert.user_id,
            email = %insert.email,
            "Saving auth credential"
        );

        sqlx::query(
            r#"
            INSERT INTO auth_credentials (id, user_id, email, password_hash)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(insert.id)
        .bind(insert.user_id)
        .bind(insert.email)
        .bind(insert.password_hash)
        .execute(&self.pool)
        .await?;

        debug!(credential_id = %insert.id, user_id = %insert.user_id, "Auth credential saved");
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
                last_login_ip = $10::inet,
                account_status = $11::account_status,
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
    async fn set_verification_token(
        &self,
        credential_id: Uuid,
        token: &str,
        expires_in_hours: i64,
    ) -> Result<(), Self::Error> {
        debug!(credential_id = %credential_id, "Setting verification token");

        sqlx::query(
            r#"
            UPDATE auth_credentials
            SET
                email_verification_token = $1,
                email_verification_expires_at = NOW() + ($2 * INTERVAL '1 hour')
            WHERE id = $3
            "#,
        )
        .bind(token)
        .bind(expires_in_hours)
        .bind(credential_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<AuthCredential>, Self::Error> {
        debug!(user_id = %user_id, "Finding auth credential by user ID");

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE user_id = $1 AND deleted_at IS NULL",
            AUTH_CREDENTIAL_COLUMNS
        );
        let row = sqlx::query_as::<_, AuthCredentialRow>(&sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<AuthCredential>, Self::Error> {
        debug!(email = %email.as_str(), "Finding auth credential by email");

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE email = $1 AND deleted_at IS NULL",
            AUTH_CREDENTIAL_COLUMNS
        );
        let row = sqlx::query_as::<_, AuthCredentialRow>(&sql)
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

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE email_verification_token = $1 AND deleted_at IS NULL AND email_verification_expires_at > NOW()",
            AUTH_CREDENTIAL_COLUMNS
        );
        let row = sqlx::query_as::<_, AuthCredentialRow>(&sql)
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

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE password_reset_token = $1 AND deleted_at IS NULL AND password_reset_expires_at > NOW()",
            AUTH_CREDENTIAL_COLUMNS
        );
        let row = sqlx::query_as::<_, AuthCredentialRow>(&sql)
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

        let sql = format!(
            "SELECT {} FROM auth_credentials WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            AUTH_CREDENTIAL_COLUMNS
        );
        let rows = sqlx::query_as::<_, AuthCredentialRow>(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn clear_expired_verification_tokens(&self) -> Result<u64, Self::Error> {
        debug!("Clearing expired email verification tokens");

        let result = sqlx::query(
            r#"
            UPDATE auth_credentials
            SET
                email_verification_token = NULL,
                email_verification_expires_at = NULL
            WHERE email_verification_expires_at IS NOT NULL
              AND email_verification_expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_credential::PasswordHash;
    use common::test_utils::TestDb;
    use std::path::Path;

    #[tokio::test]
    async fn test_save_and_find_credential() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        let email = Email::new("test@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let user_id = Uuid::new_v4(); // Dummy user_id for test
        let credential = AuthCredential::new(user_id, email.clone(), password_hash);

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

    #[tokio::test]
    async fn test_exists_methods() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        let email = Email::new("exists@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let user_id = Uuid::new_v4(); // Dummy user_id for test
        let credential = AuthCredential::new(user_id, email.clone(), password_hash);

        // Should not exist initially
        assert!(!repo.exists(credential.id()).await.unwrap());
        assert!(!repo.exists_by_email(&email).await.unwrap());

        // Save
        repo.save(&credential).await.unwrap();

        // Should exist now
        assert!(repo.exists(credential.id()).await.unwrap());
        assert!(repo.exists_by_email(&email).await.unwrap());
    }

    #[tokio::test]
    async fn test_count_and_pagination() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        // Create 5 credentials
        for i in 0..5 {
            let email = Email::new(format!("user{}@example.com", i)).unwrap();
            let password_hash = PasswordHash::from_hash("$argon2id$...");
            let user_id = Uuid::new_v4(); // Dummy user_id for test
            let credential = AuthCredential::new(user_id, email, password_hash);
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

    #[tokio::test]
    async fn test_set_and_find_verification_token() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        let email = Email::new("verify@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let user_id = Uuid::new_v4();
        let credential = AuthCredential::new(user_id, email.clone(), password_hash);
        repo.save(&credential).await.unwrap();

        let token = "test_verification_token_123";
        let expires_in_hours = 1; // 1 hour from now

        // Set token
        repo.set_verification_token(credential.id(), token, expires_in_hours)
            .await
            .unwrap();

        // Fetch updated credential
        let updated_credential = repo.find_by_id(credential.id()).await.unwrap().unwrap();
        assert_eq!(updated_credential.email_verification_token(), Some(token));
        assert!(updated_credential.email_verification_expires_at().is_some());
        assert!(!updated_credential.is_email_verified()); // Should still be false until verified

        // Find by verification token (should succeed)
        let found_by_token = repo.find_by_verification_token(token).await.unwrap();
        assert!(found_by_token.is_some());
        assert_eq!(found_by_token.unwrap().email(), &email);

        // Find by non-existent token (should fail)
        let found_by_non_existent = repo
            .find_by_verification_token("non_existent")
            .await
            .unwrap();
        assert!(found_by_non_existent.is_none());
    }

    #[tokio::test]
    async fn test_find_verification_token_expired() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        let email = Email::new("expired@example.com").unwrap();
        let password_hash = PasswordHash::from_hash("$argon2id$...");
        let user_id = Uuid::new_v4();
        let credential = AuthCredential::new(user_id, email.clone(), password_hash);
        repo.save(&credential).await.unwrap();

        let token = "expired_token_456";
        let expires_in_hours = -1; // -1 hour, already expired

        // Set token (already expired)
        repo.set_verification_token(credential.id(), token, expires_in_hours)
            .await
            .unwrap();

        // Try to find by verification token (should fail because it's expired)
        let found_expired = repo.find_by_verification_token(token).await.unwrap();
        assert!(found_expired.is_none());
    }

    #[tokio::test]
    async fn test_clear_expired_verification_tokens() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresAuthCredentialRepository::new(db.pool().clone());

        // Credential with expired token
        let email1 = Email::new("cleanme1@example.com").unwrap();
        let credential1 = AuthCredential::new(
            Uuid::new_v4(),
            email1.clone(),
            PasswordHash::from_hash("$argon2id$..."),
        );
        repo.save(&credential1).await.unwrap();
        repo.set_verification_token(credential1.id(), "token1", -1)
            .await
            .unwrap(); // Expired

        // Credential with valid token
        let email2 = Email::new("cleanme2@example.com").unwrap();
        let credential2 = AuthCredential::new(
            Uuid::new_v4(),
            email2.clone(),
            PasswordHash::from_hash("$argon2id$..."),
        );
        repo.save(&credential2).await.unwrap();
        repo.set_verification_token(credential2.id(), "token2", 1)
            .await
            .unwrap(); // Valid

        // Credential without token
        let email3 = Email::new("cleanme3@example.com").unwrap();
        let credential3 = AuthCredential::new(
            Uuid::new_v4(),
            email3.clone(),
            PasswordHash::from_hash("$argon2id$..."),
        );
        repo.save(&credential3).await.unwrap();

        // Clear expired tokens
        let cleared_count = repo.clear_expired_verification_tokens().await.unwrap();
        assert_eq!(cleared_count, 1); // Only credential1 should be cleared

        // Check credential1: token should be null
        let cred1_after_clear = repo.find_by_id(credential1.id()).await.unwrap().unwrap();
        assert!(cred1_after_clear.email_verification_token().is_none());
        assert!(cred1_after_clear.email_verification_expires_at().is_none());
        assert!(!cred1_after_clear.is_email_verified()); // Still false

        // Check credential2: token should still be valid
        let cred2_after_clear = repo.find_by_id(credential2.id()).await.unwrap().unwrap();
        assert_eq!(cred2_after_clear.email_verification_token(), Some("token2"));
        assert!(cred2_after_clear.email_verification_expires_at().is_some());

        // Check credential3: no change
        let cred3_after_clear = repo.find_by_id(credential3.id()).await.unwrap().unwrap();
        assert!(cred3_after_clear.email_verification_token().is_none());
    }
}
