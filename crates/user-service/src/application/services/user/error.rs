use thiserror::Error;
use common::persistance::error::RepositoryError;
use crate::domain::user::error::UserDomainError;

/// Application-level errors for User Service
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("User not found")]
    UserNotFound,

    #[error("Username not found: {0}")]
    UsernameNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Domain error: {0}")]
    Domain(#[from] UserDomainError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

// ─── Conversion to common::AppError (for HTTP layer) ────────────────────────
// This will be implemented when we add the presentation layer