use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::domain::user::AuthDomainError;

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
    type Err = AuthDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "user" => Ok(UserRole::User),
            _ => Err(AuthDomainError::InvalidRole(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordHashVO {
    hash: String,
}

impl PasswordHashVO {
    pub fn from_hash(hash: String) -> Self {
        Self { hash }
    }

    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }
}
