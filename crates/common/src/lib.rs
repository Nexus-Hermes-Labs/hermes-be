// Common library for Hermes
// Shared types, utilities, and infrastructure code

pub mod error;
pub mod events;
pub mod db;
pub mod cache;
pub mod message_queue;
pub mod jwt;

// Re-export commonly used types
pub use error::{AppError, Result};
pub use events::Event;

use sqlx::PgPool;
use redis::Client as RedisClient;
use async_nats::Client as NatsClient;

/// Application state shared across services
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisClient,
    pub nats: NatsClient,
}

impl AppState {
    pub async fn new(
        database_url: &str,
        redis_url: &str,
        nats_url: &str,
    ) -> Result<Self> {
        let db = PgPool::connect(database_url)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let redis = RedisClient::open(redis_url)
            .map_err(|e| AppError::Cache(e.to_string()))?;

        let nats = async_nats::connect(nats_url)
            .await
            .map_err(|e| AppError::MessageQueue(e.to_string()))?;

        Ok(Self { db, redis, nats })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = AppError::NotFound("test".to_string());
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
