use crate::domain::user::valueobject::UserStatus;
use crate::domain::user::UserRole;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Auth domain view of User - only auth-related fields
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, display_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            username,
            display_name,
            avatar_url: None,
            bio: None,
            status: UserStatus::Offline,
            role: UserRole::User,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn is_moderator(&self) -> bool {
        self.role == UserRole::Moderator
    }
}
