use std::fmt;
use std::str::FromStr;

use super::error::ReactionError;

/// Validated emoji string (1–32 chars, non-empty).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emoji(String);

impl Emoji {
    pub fn new(value: impl Into<String>) -> Result<Self, ReactionError> {
        let value = value.into();
        if value.is_empty() || value.len() > 32 {
            return Err(ReactionError::InvalidEmoji);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Emoji {
    type Err = ReactionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
