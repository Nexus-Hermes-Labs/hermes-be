use std::fmt;
use std::str::FromStr;

use super::error::ConversationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationType {
    Dm,
    GroupDm,
}

impl ConversationType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dm      => "dm",
            Self::GroupDm => "group_dm",
        }
    }
}

impl fmt::Display for ConversationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ConversationType {
    type Err = ConversationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dm"       => Ok(Self::Dm),
            "group_dm" => Ok(Self::GroupDm),
            _          => Err(ConversationError::InvalidConversationType),
        }
    }
}
