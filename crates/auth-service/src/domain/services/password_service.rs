use async_trait::async_trait;

use crate::domain::auth_credential::PasswordHash;

/// Password hashing and verification service
///
/// This is a domain service interface that abstracts password hashing
/// implementation details. The actual implementation (Argon2) lives in
/// the infrastructure layer.
#[async_trait]
pub trait PasswordService: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Hash a plaintext password
    ///
    /// # Arguments
    /// * `password` - Plaintext password to hash
    ///
    /// # Returns
    /// * `PasswordHash` value object containing the hash
    fn hash_password(&self, password: &str) -> Result<PasswordHash, Self::Error>;

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
    ) -> Result<bool, Self::Error>;
}
