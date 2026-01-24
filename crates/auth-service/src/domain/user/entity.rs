use crate::domain::user::UserRole;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Auth domain view of User - only auth-related fields
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub role: UserRole,

    // Shared fields
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            username,
            password_hash,
            role: UserRole::User,
            is_active: true,
            email_verified: false,
            email_verification_token: None,
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

    pub fn verify_email(&mut self) {
        if !self.email_verified {
            self.email_verified = true;
            self.updated_at = Utc::now();
        }
    }
}
