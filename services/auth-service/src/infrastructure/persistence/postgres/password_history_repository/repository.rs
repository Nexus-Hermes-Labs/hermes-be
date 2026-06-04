use crate::domain::password_history::{PasswordHistory, PasswordHistoryRepository};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresPasswordHistoryRepository {
    pool: PgPool,
}

impl PostgresPasswordHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordHistoryRepository for PostgresPasswordHistoryRepository {
    async fn save(&self, entry: &PasswordHistory) -> Result<(), RepositoryError> {
        debug!(credential_id = %entry.credential_id(), "Saving password history entry");

        sqlx::query(
            r"
            INSERT INTO password_history (id, credential_id, password_hash, created_at)
            VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(entry.id())
        .bind(entry.credential_id())
        .bind(entry.password_hash())
        .bind(entry.created_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_recent_by_credential(
        &self,
        credential_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PasswordHistory>, RepositoryError> {
        debug!(credential_id = %credential_id, limit = limit, "Finding recent password history");

        let rows = sqlx::query_as::<_, PasswordHistoryRow>(
            r"
            SELECT id, credential_id, password_hash, created_at
            FROM password_history
            WHERE credential_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            ",
        )
        .bind(credential_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_oldest_beyond_limit(
        &self,
        credential_id: Uuid,
        keep_count: i64,
    ) -> Result<u64, RepositoryError> {
        debug!(credential_id = %credential_id, keep_count = keep_count, "Pruning old password history");

        let result = sqlx::query(
            r"
            DELETE FROM password_history
            WHERE credential_id = $1
              AND id NOT IN (
                  SELECT id FROM password_history
                  WHERE credential_id = $1
                  ORDER BY created_at DESC
                  LIMIT $2
              )
            ",
        )
        .bind(credential_id)
        .bind(keep_count)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct PasswordHistoryRow {
    id: Uuid,
    credential_id: Uuid,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<PasswordHistoryRow> for PasswordHistory {
    fn from(row: PasswordHistoryRow) -> Self {
        PasswordHistory::from_persisted(row.id, row.credential_id, row.password_hash, row.created_at)
    }
}
