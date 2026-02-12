use thiserror::Error;
use uuid::Uuid;

/// User Privacy Settings domain errors
#[derive(Debug, Error)]
pub enum UserPrivacyError {
    // ============================================
    // PRIVACY SETTINGS ERRORS
    // ============================================
    #[error("Privacy settings not found for user {user_id}")]
    NotFound { user_id: Uuid },

    #[error("Invalid DM privacy setting: {0}")]
    InvalidDmPrivacy(String),

    #[error("Invalid friend request privacy setting: {0}")]
    InvalidFriendRequestPrivacy(String),

    #[error("Invalid content filter level: {0}")]
    InvalidContentFilterLevel(i16),

    // ============================================
    // AUTHORIZATION ERRORS (Privacy checks)
    // ============================================
    #[error("User does not accept DMs from this relationship type")]
    DmNotAllowed,

    #[error("User does not accept friend requests from this relationship type")]
    FriendRequestNotAllowed,

    // ============================================
    // GENERAL ERRORS
    // ============================================
    #[error("Repository error: {0}")]
    RepositoryError(String),
}
