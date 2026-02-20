use std::fmt;
use std::str::FromStr;

use super::error::GuildError;

// ============================================
// GUILD NAME
// ============================================

/// Guild name value object
///
/// Rules:
/// - 2-100 characters
/// - Printable characters only (no control chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GuildName(String);

impl GuildName {
    /// Validate and create a new `GuildName`.
    ///
    /// Returns [`GuildError::InvalidNameLength`] if the trimmed value is outside 2–100 characters,
    /// or [`GuildError::InvalidNameFormat`] if it contains control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, GuildError> {
        let value = value.into().trim().to_string();

        if value.len() < 2 || value.len() > 100 {
            return Err(GuildError::InvalidNameLength);
        }

        // Reject control characters
        if value.chars().any(|c| c.is_control()) {
            return Err(GuildError::InvalidNameFormat);
        }

        Ok(Self(value))
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GuildName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for GuildName {
    type Err = GuildError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// ============================================
// GUILD VISIBILITY
// ============================================

/// Whether the guild is publicly discoverable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuildVisibility {
    /// Listed in guild discovery
    Public,
    /// Not listed; join via invite only
    Private,
}

impl GuildVisibility {
    /// Returns the lowercase string representation used in the API and database (`"public"` / `"private"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    /// Returns `true` if the guild is publicly discoverable.
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }
}

impl Default for GuildVisibility {
    fn default() -> Self {
        Self::Private
    }
}

impl fmt::Display for GuildVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for GuildVisibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            other => Err(format!("Unknown guild visibility: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_name_creation() {
        assert!(GuildName::new("My Guild").is_ok());
        assert!(GuildName::new("A").is_err()); // Too short
        assert!(GuildName::new("a".repeat(101)).is_err()); // Too long
        assert!(GuildName::new("  trimmed  ").is_ok()); // Trimmed
    }

    #[test]
    fn test_guild_visibility() {
        assert_eq!(GuildVisibility::Public.as_str(), "public");
        assert!(GuildVisibility::Public.is_public());
        assert!(!GuildVisibility::Private.is_public());
        assert_eq!(GuildVisibility::default(), GuildVisibility::Private);
    }
}
