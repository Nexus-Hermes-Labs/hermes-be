use chrono::{DateTime, Utc};
use thiserror::Error;

/// Domain errors for AuthCredential
#[derive(Debug, Error)]
pub enum AuthCredentialError {
    // Email errors
    #[error("Invalid email format")]
    InvalidEmail,

    // Email verification errors
    #[error("No verification token set")]
    NoVerificationToken,

    #[error("Invalid verification token")]
    InvalidVerificationToken,

    #[error("Verification token has expired")]
    VerificationTokenExpired,

    // Password reset errors
    #[error("No password reset token set")]
    NoPasswordResetToken,

    #[error("Invalid password reset token")]
    InvalidPasswordResetToken,

    #[error("Password reset token has expired")]
    PasswordResetTokenExpired,

    // Account status errors
    #[error("Account is inactive")]
    AccountInactive,

    #[error("Account is suspended")]
    AccountSuspended,

    #[error("Account is deleted")]
    AccountDeleted,

    #[error("Account is locked until {locked_until:?}")]
    AccountLocked {
        locked_until: Option<DateTime<Utc>>,
    },

    // Invalid enum
    #[error("Invalid account status: {0}")]
    InvalidAccountStatus(String),
}
