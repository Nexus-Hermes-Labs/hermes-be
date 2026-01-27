use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database representation of User
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserEntity {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub status: UserStatusEntity,
    pub role: UserRoleEntity,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRoleEntity {
    Admin,
    Moderator,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatusEntity {
    Online,
    Offline,
    Idle,
    Dnd,
}