use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::auth_credential::{AccountStatus, AuthCredential, Email, PasswordHash};

/// Database row for auth_credentials table
#[derive(Debug, Clone, FromRow)]
pub struct AuthCredentialRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub account_status: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AuthCredentialRow> for AuthCredential {
    type Error = String;

    fn try_from(row: AuthCredentialRow) -> Result<Self, Self::Error> {
        let email = Email::new(row.email)
            .map_err(|e| format!("Invalid email in database: {:?}", e))?;

        let password_hash = PasswordHash::from_hash(row.password_hash);

        let account_status = row
            .account_status
            .parse::<AccountStatus>()
            .map_err(|e| format!("Invalid account status: {:?}", e))?;

        Ok(AuthCredential::from_persisted(
            row.id,
            email,
            password_hash,
            row.email_verified,
            row.email_verification_token,
            row.email_verification_expires_at,
            row.failed_login_attempts,
            row.locked_until,
            row.last_login_at,
            row.last_login_ip,
            account_status,
            row.deleted_at,
            row.password_reset_token,
            row.password_reset_expires_at,
            row.password_changed_at,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// Helper for INSERT operations
pub struct AuthCredentialInsert {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}

impl From<&AuthCredential> for AuthCredentialInsert {
    fn from(credential: &AuthCredential) -> Self {
        Self {
            id: credential.id(),
            email: credential.email().as_str().to_string(),
            password_hash: credential.password_hash().as_str().to_string(),
        }
    }
}

/// Helper for UPDATE operations
pub struct AuthCredentialUpdate {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub account_status: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
}

impl From<&AuthCredential> for AuthCredentialUpdate {
    fn from(credential: &AuthCredential) -> Self {
        Self {
            id: credential.id(),
            email: credential.email().as_str().to_string(),
            password_hash: credential.password_hash().as_str().to_string(),
            email_verified: credential.is_email_verified(),
            email_verification_token: credential.email_verification_token().map(|s| s.to_string()),
            email_verification_expires_at: credential.email_verification_expires_at(),
            failed_login_attempts: credential.failed_login_attempts(),
            locked_until: credential.locked_until(),
            last_login_at: credential.last_login_at(),
            last_login_ip: credential.last_login_ip().map(|s| s.to_string()),
            account_status: credential.account_status().as_str().to_string(),
            deleted_at: credential.deleted_at(),
            password_reset_token: credential.password_reset_token().map(|s| s.to_string()),
            password_reset_expires_at: credential.password_reset_expires_at(),
            password_changed_at: credential.password_changed_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_row_conversion() {
        let row = AuthCredentialRow {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "$argon2id$...".to_string(),
            email_verified: false,
            email_verification_token: None,
            email_verification_expires_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: None,
            last_login_ip: None,
            account_status: "active".to_string(),
            deleted_at: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            password_changed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let credential: AuthCredential = row.try_into().unwrap();
        assert_eq!(credential.email().as_str(), "test@example.com");
    }
}
