use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::persistence::error::RepositoryError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OutboxEventRecord {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: Value,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct OutboxRepository {
    pool: PgPool,
    source_service: String,
}

impl OutboxRepository {
    pub fn new(pool: PgPool, source_service: impl Into<String>) -> Self {
        Self {
            pool,
            source_service: source_service.into(),
        }
    }

    pub async fn fetch_publishable(
        &self,
        limit: i64,
        max_retries: i32,
    ) -> Result<Vec<OutboxEventRecord>, RepositoryError> {
        let events = sqlx::query_as::<_, OutboxEventRecord>(
            r#"
            SELECT
                id, aggregate_id, aggregate_type, event_type, payload,
                status, retry_count, last_error, created_at, published_at
            FROM outbox_events
            WHERE source_service = $1
              AND status IN ('pending', 'failed')
              AND retry_count < $2
              AND next_retry_at <= NOW()
            ORDER BY next_retry_at ASC
            LIMIT $3
            "#,
        )
        .bind(&self.source_service)
        .bind(max_retries)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    pub async fn mark_published(&self, event_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'published',
                published_at = NOW(),
                last_error = NULL
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Exponential backoff: 2^retry_count seconds, capped at 1 hour. retry_count
    // is the post-increment value, so the first failure waits 2s, the second
    // 4s, and once we hit ~11 failures every subsequent retry waits the cap.
    pub async fn mark_failed(&self, event_id: Uuid, error: &str) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'failed',
                retry_count = retry_count + 1,
                last_error = $2,
                next_retry_at = NOW()
                    + LEAST(POWER(2, retry_count + 1), 3600) * INTERVAL '1 second'
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .bind(error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
