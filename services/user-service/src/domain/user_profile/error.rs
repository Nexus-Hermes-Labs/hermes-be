use thiserror::Error;

/// User Profile domain errors
#[derive(Debug, Error)]
pub enum UserProfileError {
    // ============================================
    // USERNAME ERRORS
    // ============================================
    #[error("Username must be between 3 and 32 characters")]
    InvalidUsernameLength,

    #[error("Username can only contain lowercase letters, numbers, and underscores")]
    InvalidUsernameFormat,

    #[error("Username is already taken")]
    UsernameAlreadyTaken,

    #[error("Username cannot be changed more than once per 30 days")]
    UsernameChangeRateLimited,

    // ============================================
    // DISPLAY NAME ERRORS
    // ============================================
    #[error("Display name cannot be empty")]
    InvalidDisplayName,

    #[error("Display name is too long (max 100 characters)")]
    DisplayNameTooLong,

    // ============================================
    // PROFILE ERRORS
    // ============================================
    #[error("Bio is too long (max 500 characters)")]
    BioTooLong,

    #[error("Avatar URL is invalid (must start with http:// or https://)")]
    InvalidAvatarUrl,

    #[error("Banner URL is invalid (must start with http:// or https://)")]
    InvalidBannerUrl,

    // ============================================
    // STATUS ERRORS
    // ============================================
    #[error("Invalid user status value: {0}")]
    InvalidStatus(String),

    #[error("Custom status text is too long (max 128 characters)")]
    CustomStatusTooLong,

    #[error("Custom status emoji is too long (max 50 characters)")]
    CustomStatusEmojiTooLong,

    #[error("Custom status expiration must be in the future")]
    CustomStatusExpirationInPast,

    // ============================================
    // GENERAL ERRORS
    // ============================================
    #[error("User profile not found")]
    NotFound,

    #[error("User profile has been deleted")]
    ProfileDeleted,
}
