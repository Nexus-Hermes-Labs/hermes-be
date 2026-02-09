use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::auth_session::AuthSession;

/// Database row for auth_sessions table
#[derive(Debug, Clone, FromRow)]
pub struct AuthSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
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
            row.user_id,
            row.refresh_token_hash,
            row.ip_address,
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
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_name: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl From<&AuthSession> for AuthSessionInsert {
    fn from(session: &AuthSession) -> Self {
        Self {
            id: session.id(),
            user_id: session.user_id(),
            refresh_token_hash: session.refresh_token_hash().to_string(),
            ip_address: session.ip_address().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            device_name: session.device_name().map(|s| s.to_string()),
            expires_at: session.expires_at(),
        }
    }
}

/// Helper for UPDATE operations
pub struct AuthSessionUpdate {
    pub id: Uuid,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: DateTime<Utc>,
}

impl From<&AuthSession> for AuthSessionUpdate {
    fn from(session: &AuthSession) -> Self {
        Self {
            id: session.id(),
            is_revoked: session.is_revoked(),
            revoked_at: session.revoked_at(),
            last_used_at: session.last_used_at(),
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
            user_id: Uuid::new_v4(),
            refresh_token_hash: "hash123".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
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
