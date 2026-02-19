use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildRole;

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildRoleResponse {
    pub role_id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub color: i32,
    pub permissions: i64,
    pub position: i32,
    pub hoist: bool,
    pub mentionable: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl From<GuildRole> for GuildRoleResponse {
    fn from(r: GuildRole) -> Self {
        Self {
            role_id: r.id(),
            guild_id: r.guild_id(),
            name: r.name().to_string(),
            color: r.color().value(),
            permissions: r.permissions().bits(),
            position: r.position(),
            hoist: r.hoist(),
            mentionable: r.mentionable(),
            is_default: r.is_default(),
            created_at: r.created_at(),
        }
    }
}
