use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    User,
    Moderator,
    Admin,
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
