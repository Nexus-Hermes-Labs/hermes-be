use std::str::FromStr;
use chrono::{DateTime, Utc};
use sqlx;
use uuid::Uuid;
use crate::domain::user::{AuthDomainError, User, UserRole};
use crate::domain::user::valueobject::PasswordHashVO;


#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert flat DB row → nested domain model.
/// Returns Err when an enum column contains an unrecognised value
/// (should never happen if the DB enum and Rust enum stay in sync).
impl TryFrom<UserRow> for User {
    type Error = AuthDomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let hash = PasswordHashVO::from_hash(row.password_hash);
        let role = UserRole::from_str(&row.role)?;
        Ok(User::from_persisted(
            row.id,
            row.username,
            row.email,
            hash,
            role,
            row.is_active,
            row.email_verified,
            row.email_verification_token,
            row.created_at,
            row.updated_at,
        ))
    }
}