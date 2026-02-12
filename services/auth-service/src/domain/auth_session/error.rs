use chrono::{DateTime, Utc};
use thiserror::Error;

/// Domain errors for AuthSession
#[derive(Debug, Error)]
pub enum AuthSessionError {
    #[error("Session has been revoked at {revoked_at:?}")]
    SessionRevoked {
        revoked_at: Option<DateTime<Utc>>,
    },

    #[error("Session expired at {expired_at}")]
    SessionExpired {
        expired_at: DateTime<Utc>,
    },

    #[error("Session not found")]
    SessionNotFound,
}
