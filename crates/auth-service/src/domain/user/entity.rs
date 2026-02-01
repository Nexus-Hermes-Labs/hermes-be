use crate::domain::user::valueobject::PasswordHashVO;
use crate::domain::user::{AuthDomainError, UserRole};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Auth domain view of User - only auth-related fields
#[derive(Debug, Clone)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    password: PasswordHashVO,
    email_verified: bool,
    email_verification_token: Option<String>,
    role: UserRole,

    // Shared fields
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, password_hash: PasswordHashVO) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            email,
            username,
            password: password_hash,
            role: UserRole::User,
            is_active: true,
            email_verified: false,
            email_verification_token: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// DB reconstruction
    pub fn from_persisted(
        id: Uuid,
        username: String,
        email: String,
        password_hash: PasswordHashVO,
        role: UserRole,
        is_active: bool,
        email_verified: bool,
        email_verification_token: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password: password_hash,
            role,
            is_active,
            email_verified,
            email_verification_token,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHashVO {
        &self.password
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_email_verified(&self) -> bool {
        self.email_verified
    }

    pub fn email_verification_token(&self) -> Option<&str> {
        self.email_verification_token.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_user(&self) -> bool {
        self.role == UserRole::User
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn is_moderator(&self) -> bool {
        self.role == UserRole::Moderator
    }

    pub fn ensure_admin(&self) -> Result<(), AuthDomainError> {
        if !self.is_admin() {
            return Err(AuthDomainError::InsufficientPermissions);
        }
        Ok(())
    }

    pub fn ensure_active(&self) -> Result<(), AuthDomainError> {
        if !self.is_active {
            return Err(AuthDomainError::UserInactive);
        }
        Ok(())
    }
}
