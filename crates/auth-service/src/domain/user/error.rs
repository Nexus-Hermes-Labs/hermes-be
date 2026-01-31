use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthDomainError {
    #[error("User account is inactive")]
    UserInactive,

    #[error("Account is temporarily locked")]
    AccountLocked,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email address is already verified")]
    EmailAlreadyVerified,

    #[error("Invalid or expired email verification token")]
    InvalidEmailVerificationToken,

    #[error("Insufficient permissions")]
    InsufficientPermissions,
}
