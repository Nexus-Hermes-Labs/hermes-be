use thiserror::Error;
use common::AppError;
use common::persistance::error::RepositoryError;
use crate::domain::user::error::UserDomainError;

/// Application-level errors for User Service
#[derive(Debug, Error)]
pub enum UserApplicationError {
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

impl From<UserApplicationError> for common::AppError {
    fn from(err: UserApplicationError) -> Self {
        match err {
            UserApplicationError::UserNotFound => AppError::NotFound {
                entity_type: "User".to_string(),
            },
            UserApplicationError::UsernameNotFound(username) => AppError::NotFound {
                entity_type: format!("User with username '{}'", username),
            },
            UserApplicationError::InvalidInput(msg) => AppError::Validation(msg),
            UserApplicationError::Unauthorized(msg) => AppError::Unauthorized(msg),
            UserApplicationError::Domain(e) => AppError::Validation(e.to_string()),
            UserApplicationError::Repository(e) => {
                AppError::InternalServerError(e.to_string())
            }
            UserApplicationError::InternalServerError(msg) => {
                AppError::InternalServerError(msg)
            }
        }
    }
}