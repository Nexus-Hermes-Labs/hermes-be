use crate::application::services::authentication::error::AuthApplicationError;
use crate::domain::auth_credential::error::AuthCredentialError;
use crate::domain::auth_credential::PasswordHash;
use async_trait::async_trait;
use std::fmt::Debug;

/// Password hashing and verification service
///
/// This is a domain service interface that abstracts password hashing
/// implementation details. The actual implementation (Argon2) lives in
/// the infrastructure layer.
#[async_trait]
pub trait PasswordService: Send + Sync {
    /// Hash a plaintext password
    ///
    /// # Arguments
    /// * `password` - Plaintext password to hash
    ///
    /// # Returns
    /// * `PasswordHash` value object containing the hash
    fn hash_password(&self, password: &str) -> Result<PasswordHash, AuthCredentialError>;

    /// Verify a password against a hash
    ///
    /// # Arguments
    /// * `password` - Plaintext password to verify
    /// * `hash` - Password hash to verify against
    ///
    /// # Returns
    /// * `true` if password matches, `false` otherwise
    fn verify_password(
        &self,
        password: &str,
        hash: &PasswordHash,
    ) -> Result<bool, AuthCredentialError>;
}

/// Email delivery service abstraction.
///
/// This trait represents an application-level service responsible for
/// sending transactional emails (e.g. verification emails).
///
/// # Architectural Notes
/// - This is an abstraction boundary between the application layer and
///   the infrastructure layer.
/// - Concrete implementations (SMTP, SendGrid, SES, etc.) must live in
///   the infrastructure layer.
/// - The application layer depends only on this contract.
///
/// # Design Considerations
/// - This service should be idempotent where possible.
/// - Implementations should handle retry, logging, and provider-specific
///   error mapping internally.
/// - Network and provider errors must be mapped to `AuthApplicationError`.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to support usage in async contexts
/// and multi-threaded runtimes.
#[async_trait]
pub trait EmailService: Send + Sync + Debug {
    /// Sends an email verification message to a user.
    ///
    /// # Arguments
    /// * `to` - Recipient email address
    /// * `token` - Email verification token
    ///
    /// # Returns
    /// * `Ok(())` if the email was successfully sent
    /// * `Err(AuthApplicationError)` if sending fails
    ///
    /// # Errors
    /// Implementations should map infrastructure-level failures
    /// (SMTP errors, HTTP provider errors, timeouts, etc.)
    /// into `AuthApplicationError`.
    async fn send_verification_email(
        &self,
        to: &str,
        token: &str,
    ) -> Result<(), AuthApplicationError>;
}
