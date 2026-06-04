use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if the action is allowed and increment the counter atomically.
    /// Returns Ok(remaining) if allowed, Err if rate limit exceeded.
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: i32,
        window_seconds: i64,
    ) -> Result<RateLimitResult, RepositoryError>;

    async fn cleanup_expired(&self) -> Result<u64, RepositoryError>;
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: i32 },
    Exceeded { retry_after_seconds: i64 },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }
}
