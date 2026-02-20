use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildRole;

/// API response representing a single guild role.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildRoleResponse {
    /// Unique role identifier.
    pub role_id: Uuid,
    /// ID of the guild this role belongs to.
    pub guild_id: Uuid,
    /// Role display name.
    pub name: String,
    /// Role color as an RGB integer (0 = no color / default grey).
    pub color: i32,
    /// Permissions bitfield (combine `Permissions::*` constants with bitwise OR).
    pub permissions: i64,
    /// Display position in the role hierarchy (higher = shown first, more priority).
    pub position: i32,
    /// Whether members with this role appear in their own section in the member list.
    pub hoist: bool,
    /// Whether all members can @mention this role.
    pub mentionable: bool,
    /// `true` for the auto-created `@everyone` role that cannot be deleted.
    pub is_default: bool,
    /// Timestamp when this role was created.
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
