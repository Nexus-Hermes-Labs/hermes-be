use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;
use common::AppError;

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

    #[error("Failed to create user profile: {0}")]
    UserProfileCreationFailed(String),

    #[error("Failed to communicate with user service: {0}")]
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

    // =====================================================
    // ACCOUNT ERRORS
    // =====================================================

    #[error("Account is locked until {locked_until}")]
    AccountLocked {
        locked_until: DateTime<Utc>,
    },

    #[error("Account is suspended")]
    AccountSuspended,

    #[error("Account is deleted")]
    AccountDeleted,

}

// =====================================================
// HTTP MAPPING
// =====================================================

impl AuthApplicationError {
    pub fn status_code(&self) -> u16 {
        match self {
            // 400 Bad Request
            Self::InvalidEmail(_)
            | Self::WeakPassword => 400,

            // 401 Unauthorized
            Self::InvalidCredentials
            | Self::InvalidToken => 401,

            // 403 Forbidden
            Self::AccountDeactivated
            | Self::EmailNotVerified
            | Self::AccountSuspended
            | Self::AccountDeleted => 403,

            // 423 Locked
            Self::AccountLocked { .. } => 423,

            // 404 Not Found
            Self::UserNotFound(_)
            | Self::UserNotFoundByEmail(_) => 404,

            // 409 Conflict
            Self::EmailAlreadyExists(_)
            | Self::UsernameAlreadyExists(_) => 409,

            // 500 Internal Server Error
            Self::HashingFailed
            | Self::TokenGenerationFailed(_)
            | Self::UserProfileCreationFailed(_)
            | Self::UserServiceError(_)
            | Self::Internal(_) => 500,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidCredentials =>
                "Invalid email or password".to_string(),

            Self::AccountDeactivated =>
                "Your account has been deactivated".to_string(),

            Self::EmailNotVerified =>
                "Please verify your email before logging in".to_string(),

            Self::InvalidToken =>
                "Invalid or expired token".to_string(),

            Self::AccountLocked { locked_until } =>
                format!("Your account is locked until {}", locked_until),

            Self::AccountSuspended =>
                "Your account has been suspended".to_string(),

            Self::AccountDeleted =>
                "Your account has been deleted".to_string(),

            Self::EmailAlreadyExists(_) =>
                "This email is already registered".to_string(),

            Self::UsernameAlreadyExists(_) =>
                "This username is already taken".to_string(),

            Self::InvalidEmail(email) =>
                format!("Invalid email format: {}", email),

            Self::WeakPassword =>
                "Password must be at least 8 characters with uppercase, lowercase, and numbers".to_string(),

            Self::UserNotFound(_)
            | Self::UserNotFoundByEmail(_) =>
                "User not found".to_string(),

            // Internal / hidden
            _ =>
                "An error occurred. Please try again later.".to_string(),
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::AccountDeactivated => "ACCOUNT_DEACTIVATED",
            Self::EmailNotVerified => "EMAIL_NOT_VERIFIED",
            Self::InvalidToken => "INVALID_TOKEN",

            Self::AccountLocked { .. } => "ACCOUNT_LOCKED",
            Self::AccountSuspended => "ACCOUNT_SUSPENDED",
            Self::AccountDeleted => "ACCOUNT_DELETED",

            Self::EmailAlreadyExists(_) => "EMAIL_ALREADY_EXISTS",
            Self::UsernameAlreadyExists(_) => "USERNAME_ALREADY_EXISTS",
            Self::InvalidEmail(_) => "INVALID_EMAIL",
            Self::WeakPassword => "WEAK_PASSWORD",

            Self::UserNotFound(_)
            | Self::UserNotFoundByEmail(_) => "USER_NOT_FOUND",

            Self::HashingFailed => "PASSWORD_HASHING_FAILED",
            Self::TokenGenerationFailed(_) => "TOKEN_GENERATION_FAILED",
            Self::UserProfileCreationFailed(_) => "USER_PROFILE_CREATION_FAILED",
            Self::UserServiceError(_) => "USER_SERVICE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn should_log(&self) -> bool {
        matches!(
        self,
        Self::HashingFailed
            | Self::TokenGenerationFailed(_)
            | Self::UserProfileCreationFailed(_)
            | Self::UserServiceError(_)
            | Self::Internal(_)
    )
    }
}

// ─── Conversion to common::AppError (for HTTP layer) ────────────────────────
// This will be implemented when we add the presentation layer
impl From<AuthApplicationError> for AppError {
    fn from(error: AuthApplicationError) -> Self {
        match error {
            // Authentication Errors -> Unauthorized
            AuthApplicationError::InvalidCredentials
            | AuthApplicationError::InvalidToken => {
                AppError::Unauthorized(error.user_message())
            }

            // Account State Errors
            AuthApplicationError::AccountLocked { .. } => {
                AppError::Locked(error.user_message())
            }

            AuthApplicationError::AccountDeactivated
            | AuthApplicationError::AccountSuspended
            | AuthApplicationError::AccountDeleted
            | AuthApplicationError::EmailNotVerified => {
                AppError::Forbidden(error.user_message())
            }

            // Validation Errors
            AuthApplicationError::InvalidEmail(email) => {
                AppError::BadRequest(format!("Invalid email format: {}", email))
            }

            AuthApplicationError::WeakPassword => {
                AppError::BadRequest(error.user_message())
            }

            // Conflict Errors
            AuthApplicationError::EmailAlreadyExists(email) => {
                AppError::Conflict(format!("Email already in use: {}", email))
            }

            AuthApplicationError::UsernameAlreadyExists(username) => {
                AppError::Conflict(format!("Username already taken: {}", username))
            }

            // Not Found Errors
            AuthApplicationError::UserNotFound(id) => {
                AppError::NotFound {
                    entity_type: format!("User with ID {}", id),
                }
            }

            AuthApplicationError::UserNotFoundByEmail(email) => {
                AppError::NotFound {
                    entity_type: format!("User with email {}", email),
                }
            }

            // JWT Errors
            AuthApplicationError::TokenGenerationFailed(jwt_error) => {
                AppError::Jwt(jwt_error)
            }

            // Internal Errors
            AuthApplicationError::HashingFailed => {
                AppError::InternalServerError("Password processing failed".to_string())
            }

            AuthApplicationError::UserProfileCreationFailed(msg)
            | AuthApplicationError::UserServiceError(msg)
            | AuthApplicationError::Internal(msg) => {
                AppError::InternalServerError(msg)
            }
        }
    }
}
