use crate::domain::rate_limit::{RateLimitResult, RateLimiter};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use sqlx::PgPool;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct PgRateLimiter {
    pool: PgPool,
}

impl PgRateLimiter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RateLimiter for PgRateLimiter {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: i32,
        window_seconds: i64,
    ) -> Result<RateLimitResult, RepositoryError> {
        debug!(key = %key, max = max_requests, window = window_seconds, "Checking rate limit");

        let row = sqlx::query_as::<_, RateLimitRow>(
            r"
            INSERT INTO rate_limit_buckets (key, window_start, count, expires_at)
            VALUES (
                $1,
                date_trunc('second', NOW()) - (EXTRACT(EPOCH FROM NOW())::bigint % $3) * INTERVAL '1 second',
                1,
                date_trunc('second', NOW()) - (EXTRACT(EPOCH FROM NOW())::bigint % $3) * INTERVAL '1 second' + $3 * INTERVAL '1 second'
            )
            ON CONFLICT (key, window_start) DO UPDATE
                SET count = rate_limit_buckets.count + 1
            RETURNING count,
                      EXTRACT(EPOCH FROM (expires_at - NOW()))::bigint AS retry_after
            ",
        )
        .bind(key)
        .bind(max_requests)
        .bind(window_seconds)
        .fetch_one(&self.pool)
        .await?;

        if row.count > max_requests {
            Ok(RateLimitResult::Exceeded {
                retry_after_seconds: row.retry_after.max(1),
            })
        } else {
            Ok(RateLimitResult::Allowed {
                remaining: max_requests - row.count,
            })
        }
    }

    async fn cleanup_expired(&self) -> Result<u64, RepositoryError> {
        let result = sqlx::query("DELETE FROM rate_limit_buckets WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct RateLimitRow {
    count: i32,
    retry_after: i64,
}
