use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Published to NATS when a guild is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildCreated {
    pub guild_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub occurred_at: DateTime<Utc>,
}

/// Published to NATS when a guild is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDeleted {
    pub guild_id: Uuid,
    pub owner_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

/// Published to NATS when guild settings change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildUpdated {
    pub guild_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

// NATS subjects
pub const GUILD_CREATED: &str = "guild.created";
pub const GUILD_DELETED: &str = "guild.deleted";
pub const GUILD_UPDATED: &str = "guild.updated";
