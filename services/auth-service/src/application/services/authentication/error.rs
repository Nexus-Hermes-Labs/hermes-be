use chrono::{DateTime, Utc};
use common::infrastructure::persistence::error::RepositoryError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthApplicationError {
    // =====================================================
    // AUTHENTICATION ERRORS
    // =====================================================
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account is deactivated")]
    AccountDeactivated,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Invalid or expired token")]
    InvalidToken,

    // =====================================================
    // REGISTRATION ERRORS
    // =====================================================
    #[error("Email already in use: {0}")]
    EmailAlreadyExists(String),

    #[error("Username already taken: {0}")]
    UsernameAlreadyExists(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Password too weak")]
    WeakPassword,

    // =====================================================
    // USER SERVICE COMMUNICATION ERRORS
    // =====================================================
    #[error("Failed to create user_profile profile: {0}")]
    UserProfileCreationFailed(String),

    #[error("Failed to communicate with user_profile service: {0}")]
    UserServiceError(String),

    // =====================================================
    // NOT FOUND ERRORS
    // =====================================================
    #[error("User not found with ID: {0}")]
    UserNotFound(Uuid),

    #[error("User not found with email: {0}")]
    UserNotFoundByEmail(String),

    // =====================================================
    // JWT ERRORS
    // =====================================================
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    // =====================================================
    // INTERNAL ERRORS
    // =====================================================
    #[error("Hashing failed")]
    HashingFailed,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    // =====================================================
    // ACCOUNT ERRORS
    // =====================================================
    #[error("Account is locked until {locked_until}")]
    AccountLocked { locked_until: DateTime<Utc> },

    #[error("Account is suspended")]
    AccountSuspended,

    #[error("Account is deleted")]
    AccountDeleted,

    // =====================================================
    // PASSWORD POLICY ERRORS
    // =====================================================
    #[error("Password policy violation: {0}")]
    PasswordPolicyViolation(String),

    #[error("Password was recently used")]
    PasswordRecentlyUsed,

    // =====================================================
    // RATE LIMITING ERRORS
    // =====================================================
    #[error("Too many requests, retry after {retry_after_seconds} seconds")]
    RateLimited { retry_after_seconds: i64 },

    // =====================================================
    // USER SERVICE CLIENT ERRORS
    // =====================================================
    #[error("User service transport error: {0}")]
    UserServiceTransportError(String),

    #[error("User service invalid response: {0}")]
    UserServiceInvalidResponse(String),
}

impl From<RepositoryError> for AuthApplicationError {
    fn from(error: RepositoryError) -> Self {
        AuthApplicationError::RepositoryError(error.to_string())
    }
}
