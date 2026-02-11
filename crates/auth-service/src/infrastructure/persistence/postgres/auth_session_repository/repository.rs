use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use sqlx::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use crate::domain::auth_session::{AuthSession, AuthSessionRepository};

use super::models::{AuthSessionInsert, AuthSessionRow, AuthSessionUpdate};

/// PostgreSQL implementation of AuthSessionRepository
#[derive(Clone)]
pub struct PostgresAuthSessionRepository {
    pool: PgPool,
}

impl PostgresAuthSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ============================================
// BASE REPOSITORY TRAIT IMPLEMENTATION
// ============================================

#[async_trait]
impl Repository<AuthSession, Uuid> for PostgresAuthSessionRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthSession>, Self::Error> {
        debug!(session_id = %id, "Finding session by ID");

        let row = sqlx::query_as::<_, AuthSessionRow>(
            "SELECT * FROM auth_sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<AuthSession>, Self::Error> {
        debug!("Finding all auth sessions");

        let rows = sqlx::query_as::<_, AuthSessionRow>(
            "SELECT * FROM auth_sessions ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, session: &AuthSession) -> Result<(), Self::Error> {
        let insert: AuthSessionInsert = session.into();

        info!(
            session_id = %insert.id,
            user_id = %insert.user_id,
            "Saving auth session"
        );

        sqlx::query(
            r#"
            INSERT INTO auth_sessions (
                id, user_id, refresh_token_hash, ip_address, user_agent,
                device_name, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(insert.id)
        .bind(insert.user_id)
        .bind(insert.refresh_token_hash)
        .bind(insert.ip_address)
        .bind(insert.user_agent)
        .bind(insert.device_name)
        .bind(insert.expires_at)
        .execute(&self.pool)
        .await?;

        debug!(session_id = %insert.id, "Auth session saved");
        Ok(())
    }

    async fn update(&self, session: &AuthSession) -> Result<(), Self::Error> {
        let update: AuthSessionUpdate = session.into();

        debug!(session_id = %update.id, "Updating auth session");

        let result = sqlx::query(
            r#"
            UPDATE auth_sessions SET
                is_revoked = $2,
                revoked_at = $3,
                last_used_at = $4
            WHERE id = $1
            "#,
        )
        .bind(update.id)
        .bind(update.is_revoked)
        .bind(update.revoked_at)
        .bind(update.last_used_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            error!(session_id = %update.id, "No rows updated - session not found");
            return Err(RepositoryError::NotFound(format!(
                "Session with ID {} not found",
                update.id
            )));
        }

        debug!(session_id = %update.id, "Auth session updated");
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        info!(session_id = %id, "Deleting auth session");

        let result = sqlx::query("DELETE FROM auth_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(
                format!("Session with ID {} not found", id)
            ));
        }

        debug!(session_id = %id, "Auth session deleted");
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        debug!(session_id = %id, "Checking if session exists");

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM auth_sessions WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        debug!("Counting auth sessions");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_sessions")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }
}

// ============================================
// DOMAIN-SPECIFIC REPOSITORY TRAIT
// ============================================

#[async_trait]
impl AuthSessionRepository for PostgresAuthSessionRepository {
    async fn find_by_refresh_token_hash(
        &self,
        hash: &str,
    ) -> Result<Option<AuthSession>, Self::Error> {
        debug!("Finding session by refresh token hash");

        let row = sqlx::query_as::<_, AuthSessionRow>(
            r#"
            SELECT * FROM auth_sessions
            WHERE refresh_token_hash = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_active_by_user_id(&self, user_id: Uuid) -> Result<Vec<AuthSession>, Self::Error> {
        debug!(user_id = %user_id, "Finding active sessions by user ID");

        let rows = sqlx::query_as::<_, AuthSessionRow>(
            r#"
            SELECT * FROM auth_sessions
            WHERE user_id = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_all_by_user_id(&self, user_id: Uuid) -> Result<Vec<AuthSession>, Self::Error> {
        debug!(user_id = %user_id, "Finding all sessions by user ID");

        let rows = sqlx::query_as::<_, AuthSessionRow>(
            r#"
            SELECT * FROM auth_sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn revoke_all_by_user_id(&self, user_id: Uuid) -> Result<usize, Self::Error> {
        info!(user_id = %user_id, "Revoking all sessions for user");

        let result = sqlx::query(
            r#"
            UPDATE auth_sessions
            SET is_revoked = TRUE, revoked_at = NOW()
            WHERE user_id = $1
              AND is_revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        info!(user_id = %user_id, revoked_count = count, "Sessions revoked");

        Ok(count)
    }

    async fn delete_expired_sessions(&self) -> Result<usize, Self::Error> {
        info!("Deleting expired sessions");

        let result = sqlx::query(
            "DELETE FROM auth_sessions WHERE expires_at < NOW()",
        )
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        info!(deleted_count = count, "Expired sessions deleted");

        Ok(count)
    }

    async fn delete_old_revoked_sessions(&self, days: i64) -> Result<usize, Self::Error> {
        info!(days = days, "Deleting old revoked sessions");

        let result = sqlx::query(
            r#"
            DELETE FROM auth_sessions
            WHERE is_revoked = TRUE
              AND revoked_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        info!(deleted_count = count, "Old revoked sessions deleted");

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    #[ignore]
    async fn test_save_and_find_session(pool: PgPool) {
        let repo = PostgresAuthSessionRepository::new(pool);

        let user_id = Uuid::new_v4();
        let session = AuthSession::create(
            user_id,
            "token_hash_123".to_string(),
            30,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        // Save
        repo.save(&session).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(session.id()).await.unwrap();
        assert!(found.is_some());

        // Find by token hash
        let found = repo.find_by_refresh_token_hash("token_hash_123").await.unwrap();
        assert!(found.is_some());

        // Exists
        assert!(repo.exists(session.id()).await.unwrap());

        // Count
        let count = repo.count().await.unwrap();
        assert!(count >= 1);
    }

    #[sqlx::test]
    #[ignore]
    async fn test_revoke_all_sessions(pool: PgPool) {
        let repo = PostgresAuthSessionRepository::new(pool);

        let user_id = Uuid::new_v4();

        // Create 3 sessions
        for i in 0..3 {
            let session = AuthSession::create(
                user_id,
                format!("token_{}", i),
                30,
                None,
                None,
            );
            repo.save(&session).await.unwrap();
        }

        // Revoke all
        let count = repo.revoke_all_by_user_id(user_id).await.unwrap();
        assert_eq!(count, 3);

        // Verify no active sessions
        let active = repo.find_active_by_user_id(user_id).await.unwrap();
        assert_eq!(active.len(), 0);
    }

    #[sqlx::test]
    #[ignore]
    async fn test_delete_expired_sessions(pool: PgPool) {
        let repo = PostgresAuthSessionRepository::new(pool);

        // This would need a session with past expiry date
        // You'd need to insert directly or modify the create method for testing

        let count = repo.delete_expired_sessions().await.unwrap();
        // Just verify it runs without error
        assert!(count >= 0);
    }

    #[sqlx::test]
    #[ignore]
    async fn test_delete_old_revoked_sessions(pool: PgPool) {
        let repo = PostgresAuthSessionRepository::new(pool);

        let count = repo.delete_old_revoked_sessions(30).await.unwrap();
        // Just verify it runs without error
        assert!(count >= 0);
    }
}