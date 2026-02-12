use crate::domain::user_profile::error::UserDomainError;
use common::persistence::error::RepositoryError;
use common::AppError;
use thiserror::Error;

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

    #[error("No available discriminators for username: {0}")]
    NoAvailableDiscriminators(String),
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
            UserApplicationError::NoAvailableDiscriminators(username) => AppError::Validation(
                format!("No available discriminators for username '{}'", username),
            ),
            UserApplicationError::Repository(e) => AppError::InternalServerError(e.to_string()),
            UserApplicationError::InternalServerError(msg) => AppError::InternalServerError(msg),
        }
    }
}
