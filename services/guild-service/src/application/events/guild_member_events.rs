use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Published when a user joins a guild
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberJoined {
    pub guild_id: Uuid,
    pub user_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

/// Published when a user leaves or is kicked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberLeft {
    pub guild_id: Uuid,
    pub user_id: Uuid,
    pub reason: String, // "left", "kicked", "banned"
    pub occurred_at: DateTime<Utc>,
}

// NATS subjects
pub const MEMBER_JOINED: &str = "guild.member.joined";
pub const MEMBER_LEFT: &str = "guild.member.left";
