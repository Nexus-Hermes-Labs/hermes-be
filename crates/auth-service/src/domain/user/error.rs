use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthDomainError {
    #[error("User account is inactive")]
    UserInactive,

    #[error("Account is temporarily locked")]
    AccountLocked,

    #[error("Insufficient permissions")]
    InsufficientPermissions,
}
