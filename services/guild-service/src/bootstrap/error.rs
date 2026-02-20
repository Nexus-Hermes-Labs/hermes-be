use thiserror::Error;
use crate::presentation::error::PresentationError;

/// Errors that can occur during service startup.
#[derive(Debug, Error)]
pub enum BootstrapError {
    /// A required component was not configured before calling `build()`.
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Failed to connect to or run migrations against PostgreSQL.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Failed to establish or maintain a Redis connection.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// An infrastructure-layer component (e.g., JWT manager) failed to initialize.
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    /// The HTTP or gRPC server failed to bind or start.
    #[error("Presentation error: {0}")]
    Presentation(#[from] PresentationError),

    /// A required environment variable or configuration value is missing or invalid.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// An unexpected internal error occurred during startup.
    #[error("Internal error: {0}")]
    Internal(String),
}
