use crate::domain::user_privacy::UserPrivacyError;
use common::infrastructure::persistence::error::RepositoryError;
use uuid::Uuid;

/// Application errors for UserPrivacy service
#[derive(Debug, thiserror::Error)]
pub enum UserPrivacyServiceError {
    #[error("Privacy settings not found for user {user_id}")]
    PrivacySettingsNotFound { user_id: Uuid },

    #[error("User does not accept DMs from this relationship type")]
    DmNotAllowed,

    #[error("User does not accept friend requests from this relationship type")]
    FriendRequestNotAllowed,

    #[error("Domain error: {0}")]
    DomainError(#[from] UserPrivacyError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for UserPrivacyServiceError {
    fn from(error: RepositoryError) -> Self {
        UserPrivacyServiceError::RepositoryError(error.to_string())
    }
}
