use std::fmt;
use std::str::FromStr;

use super::error::ChannelError;

// ============================================
// CHANNEL NAME
// ============================================

/// Channel name value object
///
/// Rules:
/// - 1-100 characters
/// - Printable characters only (no control chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelName(String);

impl ChannelName {
    /// Validate and create a new `ChannelName`.
    pub fn new(value: impl Into<String>) -> Result<Self, ChannelError> {
        let value = value.into().trim().to_string();

        if value.is_empty() || value.len() > 100 {
            return Err(ChannelError::InvalidNameLength);
        }

        if value.chars().any(char::is_control) {
            return Err(ChannelError::InvalidNameFormat);
        }

        Ok(Self(value))
    }

    /// Returns the underlying string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ChannelName {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// ============================================
// CHANNEL TYPE
// ============================================

/// The type of communication a channel supports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChannelType {
    /// Text-based messaging channel
    #[default]
    Text,
    /// Voice communication channel
    Voice,
    /// Organizational category container
    Category,
    /// Read-only announcement channel
    Announcement,
}

impl ChannelType {
    /// Returns the lowercase string representation used in the API and database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Voice => "voice",
            Self::Category => "category",
            Self::Announcement => "announcement",
        }
    }

    /// Returns `true` if this is a category channel (can act as parent).
    #[must_use]
    pub const fn is_category(self) -> bool {
        matches!(self, Self::Category)
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChannelType {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "voice" => Ok(Self::Voice),
            "category" => Ok(Self::Category),
            "announcement" => Ok(Self::Announcement),
            other => Err(ChannelError::InvalidChannelType(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name_creation() {
        assert!(ChannelName::new("general").is_ok());
        assert!(ChannelName::new("").is_err());
        assert!(ChannelName::new("a".repeat(101)).is_err());
        assert!(ChannelName::new("  trimmed  ").is_ok());
    }

    #[test]
    fn test_channel_type_parsing() {
        assert_eq!("text".parse::<ChannelType>().unwrap(), ChannelType::Text);
        assert_eq!(
            "voice".parse::<ChannelType>().unwrap(),
            ChannelType::Voice
        );
        assert_eq!(
            "category".parse::<ChannelType>().unwrap(),
            ChannelType::Category
        );
        assert_eq!(
            "announcement".parse::<ChannelType>().unwrap(),
            ChannelType::Announcement
        );
        assert!("invalid".parse::<ChannelType>().is_err());
    }

    #[test]
    fn test_channel_type_as_str() {
        assert_eq!(ChannelType::Text.as_str(), "text");
        assert!(ChannelType::Category.is_category());
        assert!(!ChannelType::Text.is_category());
    }
}
