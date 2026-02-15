use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for auth_audit_log table
#[derive(Debug, Clone, FromRow)]
pub struct AuthAuditLogRow {
    pub id: Uuid,
    pub credential_id: Uuid,
    pub event_type: String,
    pub event_description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

/// Read-only audit log entry
#[derive(Debug, Clone)]
pub struct AuthAuditLog {
    pub id: Uuid,
    pub credential_id: Uuid,
    pub event_type: String,
    pub event_description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

impl From<AuthAuditLogRow> for AuthAuditLog {
    fn from(row: AuthAuditLogRow) -> Self {
        Self {
            id: row.id,
            credential_id: row.credential_id,
            event_type: row.event_type,
            event_description: row.event_description,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

/// Audit log query filters
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilters {
    pub credential_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
