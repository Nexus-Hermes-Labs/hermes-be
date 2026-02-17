use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

use crate::domain::auth_credential::{PasswordHash as DomainPasswordHash, PasswordService, AuthCredentialError};

// ============================================
// ARGON2 PASSWORD SERVICE
// ============================================

/// Argon2 password hashing service
///
/// Uses Argon2id variant which provides balanced protection against
/// side-channel and GPU attacks.
///
/// Security parameters (default):
/// - Memory cost: 19 MiB
/// - Time cost: 2 iterations
/// - Parallelism: 1 thread
///
/// These are the OWASP recommended minimum values for Argon2id.
#[derive(Clone)]
pub struct Argon2PasswordService {
    hasher: Argon2<'static>,
}

impl Argon2PasswordService {
    /// Create a new Argon2 password service with default parameters
    pub fn new() -> Self {
        Self {
            hasher: Argon2::default(),
        }
    }

    /// Create with custom Argon2 configuration
    ///
    /// Use this for fine-tuning security vs performance trade-offs.
    /// For most use cases, `new()` with default parameters is recommended.
    pub fn with_config(hasher: Argon2<'static>) -> Self {
        Self { hasher }
    }
}

impl Default for Argon2PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordService for Argon2PasswordService {
    fn hash_password(&self, password: &str) -> Result<DomainPasswordHash, AuthCredentialError> {
        // Validate password length
        if password.is_empty() {
            return Err(AuthCredentialError::EmptyPassword);
        }

        if password.len() > 72 {
            // Argon2 has a max length of 4294967295 bytes, but 72 is a practical limit
            return Err(AuthCredentialError::PasswordTooLong);
        }

        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password
        let password_hash = self
            .hasher
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthCredentialError::HashingFailed(e.to_string()))?;

        // Convert to PHC string format
        // Format: $argon2id$v=19$m=19456,t=2,p=1$SALT$HASH
        Ok(DomainPasswordHash::from_hash(password_hash.to_string()))
    }

    fn verify_password(
        &self,
        password: &str,
        hash: &DomainPasswordHash,
    ) -> Result<bool, AuthCredentialError> {
        // Parse the stored hash
        let parsed_hash = PasswordHash::new(hash.as_str())
            .map_err(|e| AuthCredentialError::InvalidHashFormat(e.to_string()))?;

        // Verify the password
        match self.hasher.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthCredentialError::VerificationFailed(e.to_string())),
        }
    }
}

// ============================================
// ERRORS
// ============================================

#[derive(Debug, Error)]
pub enum PasswordServiceError {
    #[error("Password cannot be empty")]
    EmptyPassword,

    #[error("Password too long (max 72 characters)")]
    PasswordTooLong,

    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Password verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid hash format: {0}")]
    InvalidHashFormat(String),
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let service = Argon2PasswordService::new();
        let password = "SecurePassword123!";

        // Hash password
        let hash = service.hash_password(password).unwrap();

        // Verify correct password
        assert!(service.verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_hash_format() {
        let service = Argon2PasswordService::new();
        let hash = service.hash_password("test_password").unwrap();

        // Check PHC format: $argon2id$v=19$...
        assert!(hash.as_str().starts_with("$argon2id$v=19$"));
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let service = Argon2PasswordService::new();
        let password = "same_password";

        let hash1 = service.hash_password(password).unwrap();
        let hash2 = service.hash_password(password).unwrap();

        // Different hashes (different salts)
        assert_ne!(hash1.as_str(), hash2.as_str());

        // Both verify correctly
        assert!(service.verify_password(password, &hash1).unwrap());
        assert!(service.verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_empty_password_rejected() {
        let service = Argon2PasswordService::new();
        let result = service.hash_password("");
        assert!(result.is_err());
    }

    #[test]
    fn test_very_long_password_rejected() {
        let service = Argon2PasswordService::new();
        let long_password = "a".repeat(73);
        let result = service.hash_password(&long_password);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hash_format() {
        let service = Argon2PasswordService::new();
        let invalid_hash = DomainPasswordHash::from_hash("invalid_hash_format");
        let result = service.verify_password("password", &invalid_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_timing_safety() {
        // This test ensures that verification time is roughly constant
        // regardless of password correctness (helps prevent timing attacks)
        let service = Argon2PasswordService::new();
        let hash = service.hash_password("correct_password").unwrap();

        use std::time::Instant;

        // Time correct password
        let start = Instant::now();
        let _ = service.verify_password("correct_password", &hash);
        let correct_duration = start.elapsed();

        // Time incorrect password
        let start = Instant::now();
        let _ = service.verify_password("wrong_password", &hash);
        let wrong_duration = start.elapsed();

        // Durations should be roughly similar (within 50ms)
        // Note: This is a basic check, not cryptographically rigorous
        let diff = if correct_duration > wrong_duration {
            correct_duration - wrong_duration
        } else {
            wrong_duration - correct_duration
        };

        assert!(diff.as_millis() < 50, "Timing difference too large: {:?}", diff);
    }
}
