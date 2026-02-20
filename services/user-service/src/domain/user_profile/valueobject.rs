use std::fmt;
use std::str::FromStr;

use super::error::UserProfileError;

// ============================================
// USERNAME (Unique identifier)
// ============================================

/// Username value object
///
/// Rules:
/// - 3-32 characters
/// - Lowercase letters, numbers, underscore only
/// - No spaces, no special characters
/// - Auto-lowercased
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn new(value: impl Into<String>) -> Result<Self, UserProfileError> {
        let value = value.into().to_lowercase(); // Auto-lowercase

        // Validate length
        if value.len() < 3 || value.len() > 32 {
            return Err(UserProfileError::InvalidUsernameLength);
        }

        // Validate format (lowercase letters, numbers, underscore)
        if !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(UserProfileError::InvalidUsernameFormat);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get mention format: "@username"
    pub fn mention(&self) -> String {
        format!("@{}", self.0)
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Username {
    type Err = UserProfileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// ============================================
// USER STATUS (Presence)
// ============================================

/// User presence status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UserStatus {
    /// User is online and active
    Online,

    /// User is away/idle
    Idle,

    /// Do not disturb (no notifications)
    DoNotDisturb,

    /// User is offline
    #[default]
    Offline,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Idle => "idle",
            Self::DoNotDisturb => "dnd",
            Self::Offline => "offline",
        }
    }

    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Online | Self::Idle)
    }
}

impl FromStr for UserStatus {
    type Err = UserProfileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "online" => Ok(Self::Online),
            "idle" => Ok(Self::Idle),
            "dnd" => Ok(Self::DoNotDisturb),
            "offline" => Ok(Self::Offline),
            _ => Err(UserProfileError::InvalidStatus(s.to_string())),
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_creation() {
        // Valid usernames
        assert!(Username::new("johndoe").is_ok());
        assert!(Username::new("user_123").is_ok());
        assert!(Username::new("test_user_2024").is_ok());

        // Auto-lowercase
        let username = Username::new("JohnDoe").unwrap();
        assert_eq!(username.as_str(), "johndoe");
    }

    #[test]
    fn test_username_validation() {
        // Too short
        assert!(Username::new("ab").is_err());

        // Too long
        assert!(Username::new("a".repeat(33)).is_err());

        // Invalid characters
        assert!(Username::new("john doe").is_err()); // Space
        assert!(Username::new("john@doe").is_err()); // Special char
        assert!(Username::new("john-doe").is_err()); // Hyphen (not allowed)
    }

    #[test]
    fn test_username_mention() {
        let username = Username::new("johndoe").unwrap();
        assert_eq!(username.mention(), "@johndoe");
    }

    #[test]
    fn test_user_status() {
        assert_eq!(UserStatus::Online.as_str(), "online");
        assert!(UserStatus::Online.is_online());
        assert!(UserStatus::Online.is_available());
        assert!(!UserStatus::DoNotDisturb.is_available());
    }

    #[test]
    fn test_user_status_parsing() {
        assert_eq!("online".parse::<UserStatus>().unwrap(), UserStatus::Online);
        assert_eq!("idle".parse::<UserStatus>().unwrap(), UserStatus::Idle);
        assert_eq!(
            "dnd".parse::<UserStatus>().unwrap(),
            UserStatus::DoNotDisturb
        );
        assert_eq!(
            "offline".parse::<UserStatus>().unwrap(),
            UserStatus::Offline
        );
        assert!("invalid".parse::<UserStatus>().is_err());
    }
}
