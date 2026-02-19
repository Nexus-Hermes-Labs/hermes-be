use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Published when a role is created, updated, or deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleChanged {
    pub guild_id: Uuid,
    pub role_id: Uuid,
    pub action: String, // "created", "updated", "deleted"
    pub occurred_at: DateTime<Utc>,
}

// NATS subjects
pub const ROLE_CHANGED: &str = "guild.role.changed";
