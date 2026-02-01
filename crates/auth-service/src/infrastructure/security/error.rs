use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraSecurityError {
    #[error("Password hashing failed")]
    HashingFailed,

    #[error("Password verification failed")]
    VerificationFailed,

    #[error("Invalid password hash format")]
    InvalidHashFormat,

    #[error("Security configuration error")]
    ConfigError,
}
