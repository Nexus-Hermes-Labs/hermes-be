use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Friend,
    Blocked,
    PendingIncoming,
    #[default]
    PendingOutgoing,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Friend => "friend",
            Self::Blocked => "blocked",
            Self::PendingIncoming => "pending_incoming",
            Self::PendingOutgoing => "pending_outgoing",
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid relationship type")]
pub struct ParseRelationshipTypeError;

impl FromStr for RelationshipType {
    type Err = ParseRelationshipTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "friend" => Ok(Self::Friend),
            "blocked" => Ok(Self::Blocked),
            "pending_incoming" => Ok(Self::PendingIncoming),
            "pending_outgoing" => Ok(Self::PendingOutgoing),
            _ => Err(ParseRelationshipTypeError),
        }
    }
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
