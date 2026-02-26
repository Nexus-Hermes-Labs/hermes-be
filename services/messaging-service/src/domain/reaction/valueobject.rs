use super::error::ReactionError;
use std::fmt;

/// Validated emoji (1–32 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
