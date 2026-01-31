use crate::domain::user::{AuthDomainError, UserRole};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::user::valueobject::PasswordHashVO;

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
    pub fn new(username: String, email: String, password_plain: String) -> Result<Self, AuthDomainError> {
        let now = Utc::now();
        let password = PasswordHashVO::new(&password_plain)?;
        Ok(Self {
            id: Uuid::new_v4(),
            email,
            username,
            password,
            role: UserRole::User,
            is_active: true,
            email_verified: false,
            email_verification_token: None,
            created_at: now,
            updated_at: now,
        })
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

    pub fn generate_

    pub fn generate_email_verification_token(&mut self) {
        let token = Uuid::new_v4().to_string();
        self.email_verification_token = Some(token);
        self.updated_at = Utc::now();
    }

    pub fn verify_email_with_token(&mut self, provided: &str) -> Result<(), AuthDomainError> {
        let current = self
            .email_verification_token
            .as_deref()
            .ok_or(AuthDomainError::InvalidVerificationToken)?;

        if current != provided {
            return Err(AuthDomainError::InvalidVerificationToken);
        }

        self.email_verified = true;
        self.email_verification_token = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn ensure_admin(&self) -> Result<(), AuthDomainError> {
        if !self.is_admin() {
            return Err(AuthDomainError::InsufficientPermissions);
        }
        Ok(())
    }

    pub fn verify_email(&mut self) -> Result<(), AuthDomainError> {
        if self.email_verified {
            return Err(AuthDomainError::EmailAlreadyVerified);
        }

        self.email_verified = true;
        self.email_verification_token = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn ensure_active(&self) -> Result<(), AuthDomainError> {
        if !self.is_active {
            return Err(AuthDomainError::UserInactive);
        }
        Ok(())
    }
}
