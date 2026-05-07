use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::auth_session::AuthSession;

/// Database row for auth_sessions table
#[derive(Debug, Clone, FromRow)]
pub struct AuthSessionRow {
    pub id: Uuid,
    pub credential_id: Uuid,
    pub refresh_token_hash: String,
    pub previous_refresh_token_hash: Option<String>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub last_used_ip: Option<String>,
    pub user_agent: Option<String>,
    pub device_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<AuthSessionRow> for AuthSession {
    type Error = String;

    fn try_from(row: AuthSessionRow) -> Result<Self, Self::Error> {
        Ok(AuthSession::from_persisted(
            row.id,
            row.credential_id,
            row.refresh_token_hash,
            row.previous_refresh_token_hash,
            row.rotated_at,
            row.ip_address,
            row.last_used_ip,
            row.user_agent,
            row.device_name,
            row.expires_at,
            row.last_used_at,
            row.is_revoked,
            row.revoked_at,
            row.created_at,
        ))
    }
}

/// Helper for INSERT operations
pub struct AuthSessionInsert {
    pub id: Uuid,
    pub credential_id: Uuid,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
    pub last_used_ip: Option<String>,
    pub user_agent: Option<String>,
    pub device_name: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl From<&AuthSession> for AuthSessionInsert {
    fn from(session: &AuthSession) -> Self {
        Self {
            id: session.id(),
            credential_id: session.credential_id(),
            refresh_token_hash: session.refresh_token_hash().to_string(),
            ip_address: session.ip_address().map(|s| s.to_string()),
            last_used_ip: session.last_used_ip().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            device_name: session.device_name().map(|s| s.to_string()),
            expires_at: session.expires_at(),
        }
    }
}

/// Helper for UPDATE operations.
///
/// All fields a session can mutate after creation are written here so the
/// same `update` query covers `mark_as_used`, `revoke`, and `rotate`.
pub struct AuthSessionUpdate {
    pub id: Uuid,
    pub refresh_token_hash: String,
    pub previous_refresh_token_hash: Option<String>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub last_used_ip: Option<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<&AuthSession> for AuthSessionUpdate {
    fn from(session: &AuthSession) -> Self {
        Self {
            id: session.id(),
            refresh_token_hash: session.refresh_token_hash().to_string(),
            previous_refresh_token_hash: session
                .previous_refresh_token_hash()
                .map(|s| s.to_string()),
            rotated_at: session.rotated_at(),
            expires_at: session.expires_at(),
            last_used_at: session.last_used_at(),
            last_used_ip: session.last_used_ip().map(|s| s.to_string()),
            is_revoked: session.is_revoked(),
            revoked_at: session.revoked_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_row_conversion() {
        let row = AuthSessionRow {
            id: Uuid::new_v4(),
            credential_id: Uuid::new_v4(),
            refresh_token_hash: "hash123".to_string(),
            previous_refresh_token_hash: None,
            rotated_at: None,
            ip_address: Some("127.0.0.1".to_string()),
            last_used_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            device_name: Some("Chrome".to_string()),
            expires_at: Utc::now(),
            last_used_at: Utc::now(),
            is_revoked: false,
            revoked_at: None,
            created_at: Utc::now(),
        };

        let session: AuthSession = row.try_into().unwrap();
        assert!(!session.is_revoked());
    }
}
