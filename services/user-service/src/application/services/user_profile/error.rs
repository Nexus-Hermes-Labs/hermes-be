use common::infrastructure::persistence::error::RepositoryError;
use crate::domain::user_profile::UserProfileError;

/// Application errors for UserProfile service
#[derive(Debug, thiserror::Error)]
pub enum UserProfileServiceError {
    #[error("Profile not found")]
    ProfileNotFound,

    #[error("Username is already taken")]
    UsernameAlreadyTaken,

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] UserProfileError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for UserProfileServiceError {
    fn from(error: RepositoryError) -> Self {
        UserProfileServiceError::RepositoryError(error.to_string())
    }
}
