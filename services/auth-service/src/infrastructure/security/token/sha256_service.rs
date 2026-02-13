use sha2::{Digest, Sha256};
use crate::domain::auth_session::{RefreshTokenHash, TokenHasher};
use crate::infrastructure::security::token::error::TokenHasherError;
// ============================================
// SHA256 TOKEN HASHER
// ============================================

/// SHA-256 based refresh token hasher
///
/// Purpose:
/// - Prevent plaintext refresh tokens from being stored
/// - Protect against database leaks
///
/// Notes:
/// - Fast by design (used on every refresh)
/// - Not intended for password hashing
#[derive(Clone)]
pub struct Sha256TokenHasher;

impl Sha256TokenHasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256TokenHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenHasher for Sha256TokenHasher {
    type Error = TokenHasherError;

    fn hash(&self, token: &str) -> Result<RefreshTokenHash, Self::Error> {
        if token.is_empty() {
            return Err(TokenHasherError::EmptyToken);
        }

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());

        Ok(RefreshTokenHash::new(format!(
            "{:x}",
            hasher.finalize()
        )))
    }
}

