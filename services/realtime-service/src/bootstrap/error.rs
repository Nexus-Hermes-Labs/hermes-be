//! Bootstrap errors for the realtime service.

use thiserror::Error;

/// Errors that can occur during service startup.
#[derive(Debug, Error)]
pub enum BootstrapError {
    /// Generic initialisation failure.
    #[error("Initialisation error: {0}")]
    Initialization(String),

    /// Redis connection or operation error.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Infrastructure-layer error (NATS, JWT, etc.).
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    /// HTTP/gRPC presentation-layer error.
    #[error("Presentation error: {0}")]
    Presentation(String),

    /// Unexpected internal error (e.g. task panic).
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<common::infrastructure::security::jwt_manager::JwtError> for BootstrapError {
    fn from(e: common::infrastructure::security::jwt_manager::JwtError) -> Self {
        Self::Infrastructure(format!("JWT manager error: {e}"))
    }
}

impl From<common_config::error::ConfigError> for BootstrapError {
    fn from(e: common_config::error::ConfigError) -> Self {
        Self::Initialization(format!("Config error: {e}"))
    }
}

impl From<anyhow::Error> for BootstrapError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
