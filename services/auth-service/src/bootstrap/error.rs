use crate::presentation::error::PresentationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Presentation error: {0}")]
    Presentation(#[from] PresentationError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
