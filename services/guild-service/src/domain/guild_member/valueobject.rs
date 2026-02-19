use std::fmt;

use super::error::GuildMemberError;

/// Member nickname value object
///
/// Optional server-specific display name override.
/// Rules: 1-32 characters, no control chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberNickname(String);

impl MemberNickname {
    pub fn new(value: impl Into<String>) -> Result<Self, GuildMemberError> {
        let value = value.into().trim().to_string();

        if value.is_empty() || value.len() > 32 {
            return Err(GuildMemberError::InvalidNicknameLength);
        }

        if value.chars().any(|c| c.is_control()) {
            return Err(GuildMemberError::InvalidNicknameFormat);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemberNickname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nickname_validation() {
        assert!(MemberNickname::new("gamer").is_ok());
        assert!(MemberNickname::new("").is_err());
        assert!(MemberNickname::new("a".repeat(33)).is_err());
    }
}
