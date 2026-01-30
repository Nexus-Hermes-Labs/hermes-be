use crate::domain::user::User;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Moderator,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
        }
    }
}

impl FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "user" => Ok(UserRole::User),
            _ => Err("Invalid role".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Online,
    Offline,
    Idle,
    Dnd,
}

impl FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "online" => Ok(UserStatus::Online),
            "offline" => Ok(UserStatus::Offline),
            "idle" => Ok(UserStatus::Idle),
            "dnd" => Ok(UserStatus::Dnd),
            _ => Err("Invalid status".to_string()),
        }
    }
}

/// UserSummary - Immutable snapshot for display
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSummary {
    pub id: uuid::Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl UserSummary {
    pub fn from_user(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
        }
    }

    /// Get user tag (username#discriminator)
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username, self.discriminator)
    }

    pub fn display_name(&self) ->String {
        self.display_name.clone()
    }
}
