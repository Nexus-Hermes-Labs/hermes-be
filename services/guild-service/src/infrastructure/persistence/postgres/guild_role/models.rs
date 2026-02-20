use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::guild_role::{GuildRole, GuildRoleError, Permissions, RoleColor};

/// Database model for `guild_roles` table
#[derive(Debug, Clone, FromRow)]
pub struct GuildRoleRow {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub color: i32,
    pub permissions: i64,
    pub position: i32,
    pub hoist: bool,
    pub mentionable: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<GuildRoleRow> for GuildRole {
    type Error = GuildRoleError;

    fn try_from(row: GuildRoleRow) -> Result<Self, Self::Error> {
        Ok(Self::from_persisted(
            row.id,
            row.guild_id,
            row.name,
            RoleColor(row.color),
            Permissions(row.permissions),
            row.position,
            row.hoist,
            row.mentionable,
            row.is_default,
            row.created_at,
            row.updated_at,
        ))
    }
}

impl From<&GuildRole> for GuildRoleRow {
    fn from(role: &GuildRole) -> Self {
        Self {
            id: role.id(),
            guild_id: role.guild_id(),
            name: role.name().to_string(),
            color: role.color().value(),
            permissions: role.permissions().bits(),
            position: role.position(),
            hoist: role.hoist(),
            mentionable: role.mentionable(),
            is_default: role.is_default(),
            created_at: role.created_at(),
            updated_at: role.updated_at(),
        }
    }
}
