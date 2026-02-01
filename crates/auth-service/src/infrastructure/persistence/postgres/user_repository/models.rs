use chrono::{DateTime, Utc};
use sqlx;
use uuid::Uuid;
use crate::domain::user::{User, UserRole};
use crate::domain::user::valueobject::PasswordHashVO;

/// Postgres user_role enum'u için dedicated type
#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRoleEntity {
    User,
    Moderator,
    Admin,
}

impl From<&UserRole> for UserRoleEntity {
    fn from(role: &UserRole) -> Self {
        match role {
            UserRole::User => Self::User,
            UserRole::Moderator => Self::Moderator,
            UserRole::Admin => Self::Admin,
        }
    }
}

impl From<UserRoleEntity> for UserRole {
    fn from(role: UserRoleEntity) -> Self {
        match role {
            UserRoleEntity::User => Self::User,
            UserRoleEntity::Moderator => Self::Moderator,
            UserRoleEntity::Admin => Self::Admin,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuthUserEntity {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRoleEntity,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DB → Domain
impl From<AuthUserEntity> for User {
    fn from(entity: AuthUserEntity) -> Self {
        User::from_persisted(
            entity.id,
            entity.username,
            entity.email,
            PasswordHashVO::from_hash(entity.password_hash),
            entity.role.into(),
            entity.is_active,
            entity.email_verified,
            entity.email_verification_token,
            entity.created_at,
            entity.updated_at,
        )
    }
}

/// Domain → DB
impl From<&User> for AuthUserEntity {
    fn from(user: &User) -> Self {
        Self {
            id: user.id(),
            username: user.username().to_string(),
            email: user.email().to_string(),
            password_hash: user.password_hash().get_hash().to_string(),
            role: user.role().into(),
            is_active: user.is_active(),
            email_verified: user.is_email_verified(),
            email_verification_token: user
                .email_verification_token()
                .map(String::from),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}