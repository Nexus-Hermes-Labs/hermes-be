use std::fmt;

use rand::{distributions::Alphanumeric, Rng};

/// Invite code value object
///
/// An 8-character alphanumeric code (case-sensitive).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InviteCode(String);

impl InviteCode {
    /// Generate a new random invite code
    pub fn generate() -> Self {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        Self(code)
    }

    /// Create from existing string (used when loading from DB)
    pub fn from_existing(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Returns the underlying code string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InviteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = InviteCode::generate();
        assert_eq!(code.as_str().len(), 8);

        // Codes should be unique (probabilistically)
        let code2 = InviteCode::generate();
        assert_ne!(code.as_str(), code2.as_str());
    }
}
