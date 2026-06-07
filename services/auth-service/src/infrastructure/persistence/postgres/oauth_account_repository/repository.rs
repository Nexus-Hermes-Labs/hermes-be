use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use sqlx::PgPool;
use tracing::debug;

use crate::domain::oauth_account::{OAuthAccount, OAuthAccountRepository, OAuthProvider};

use super::models::OAuthAccountRow;

const OAUTH_ACCOUNT_COLUMNS: &str =
    "id, credential_id, provider, provider_user_id, email, created_at";

/// PostgreSQL implementation of [`OAuthAccountRepository`] (reads only; writes
/// go through the auth Unit of Work).
#[derive(Clone)]
pub struct PostgresOAuthAccountRepository {
    pool: PgPool,
}

impl PostgresOAuthAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthAccountRepository for PostgresOAuthAccountRepository {
    async fn find_by_provider_and_subject(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, RepositoryError> {
        debug!(provider = %provider, "Finding oauth account by provider + subject");

        let sql = format!(
            "SELECT {OAUTH_ACCOUNT_COLUMNS} FROM oauth_accounts \
             WHERE provider = $1 AND provider_user_id = $2"
        );
        let row = sqlx::query_as::<_, OAuthAccountRow>(&sql)
            .bind(provider.as_str())
            .bind(provider_user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_credential::{AuthCredential, Email, PasswordHash};
    use crate::infrastructure::persistence::postgres::PostgresAuthCredentialRepository;
    use common::infrastructure::persistence::repository::Repository;
    use common::test_utils::TestDb;
    use std::path::Path;
    use uuid::Uuid;

    async fn seed_credential(pool: &PgPool) -> Uuid {
        let repo = PostgresAuthCredentialRepository::new(pool.clone());
        let email = Email::new("oauth-user@example.com").unwrap();
        let credential =
            AuthCredential::new(Uuid::new_v4(), email, PasswordHash::from_hash("$argon2id$..."));
        repo.save(&credential).await.unwrap();
        credential.id()
    }

    #[tokio::test]
    async fn test_find_by_provider_and_subject() {
        let db = TestDb::new(Path::new("migrations")).await;
        let pool = db.pool().clone();
        let credential_id = seed_credential(&pool).await;

        // Insert a link directly (writer path is exercised via UoW tests).
        sqlx::query(
            "INSERT INTO oauth_accounts (credential_id, provider, provider_user_id, email) \
             VALUES ($1, 'google', 'sub-123', 'oauth-user@example.com')",
        )
        .bind(credential_id)
        .execute(&pool)
        .await
        .unwrap();

        let repo = PostgresOAuthAccountRepository::new(pool);

        let found = repo
            .find_by_provider_and_subject(OAuthProvider::Google, "sub-123")
            .await
            .unwrap();
        assert!(found.is_some());
        let account = found.unwrap();
        assert_eq!(account.credential_id(), credential_id);
        assert_eq!(account.provider(), OAuthProvider::Google);

        let missing = repo
            .find_by_provider_and_subject(OAuthProvider::Google, "does-not-exist")
            .await
            .unwrap();
        assert!(missing.is_none());
    }
}
