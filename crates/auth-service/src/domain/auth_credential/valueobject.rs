use std::fmt;
use std::str::FromStr;

use super::error::AuthCredentialError;

// ============================================
// EMAIL VALUE OBJECT
// ============================================

/// Email address value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    /// Create new email with validation
    pub fn new(email: impl Into<String>) -> Result<Self, AuthCredentialError> {
        let email = email.into();

        // Basic email validation
        if !email.contains('@') || email.len() > 255 {
            return Err(AuthCredentialError::InvalidEmail);
        }

        // Simple regex check
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();

        if !email_regex.is_match(&email) {
            return Err(AuthCredentialError::InvalidEmail);
        }

        Ok(Self(email.to_lowercase()))
    }

    /// Get email as string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Email {
    type Err = AuthCredentialError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// ============================================
// PASSWORD HASH VALUE OBJECT
// ============================================

/// Password hash value object (Argon2 hash)
#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Create from existing hash (from database)
    pub fn from_hash(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Get hash as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get hash as owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// ============================================
// ACCOUNT STATUS ENUM
// ============================================

/// Account status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    Active,
    Suspended,
    Deleted,
}

impl AccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_suspended(&self) -> bool {
        matches!(self, Self::Suspended)
    }

    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted)
    }
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AccountStatus {
    type Err = AuthCredentialError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "deleted" => Ok(Self::Deleted),
            _ => Err(AuthCredentialError::InvalidAccountStatus(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_email_lowercase() {
        let email = Email::new("Test@Example.COM").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        let result = Email::new("notanemail");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(250));
        let result = Email::new(long_email);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_redacted() {
        let hash = PasswordHash::from_hash("$argon2id$v=19$...");
        assert_eq!(format!("{}", hash), "[REDACTED]");
    }

    #[test]
    fn test_account_status_from_str() {
        assert_eq!(
            "active".parse::<AccountStatus>().unwrap(),
            AccountStatus::Active
        );
        assert_eq!(
            "SUSPENDED".parse::<AccountStatus>().unwrap(),
            AccountStatus::Suspended
        );
        assert!("invalid".parse::<AccountStatus>().is_err());
    }

    #[test]
    fn test_account_status_as_str() {
        assert_eq!(AccountStatus::Active.as_str(), "active");
        assert_eq!(AccountStatus::Suspended.as_str(), "suspended");
        assert_eq!(AccountStatus::Deleted.as_str(), "deleted");
    }
}
