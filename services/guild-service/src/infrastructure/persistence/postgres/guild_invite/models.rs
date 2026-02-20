use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::guild_invite::{GuildInvite, GuildInviteError, InviteCode};

/// Database model for `guild_invites` table
#[derive(Debug, Clone, FromRow)]
pub struct GuildInviteRow {
    pub code: String,
    pub guild_id: Uuid,
    pub creator_id: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<GuildInviteRow> for GuildInvite {
    type Error = GuildInviteError;

    fn try_from(row: GuildInviteRow) -> Result<Self, Self::Error> {
        Ok(Self::from_persisted(
            InviteCode::from_existing(row.code),
            row.guild_id,
            row.creator_id,
            row.max_uses,
            row.uses,
            row.expires_at,
            row.revoked,
            row.created_at,
        ))
    }
}

impl From<&GuildInvite> for GuildInviteRow {
    fn from(invite: &GuildInvite) -> Self {
        Self {
            code: invite.code().as_str().to_string(),
            guild_id: invite.guild_id(),
            creator_id: invite.creator_id(),
            max_uses: invite.max_uses(),
            uses: invite.uses(),
            expires_at: invite.expires_at(),
            revoked: invite.is_revoked(),
            created_at: invite.created_at(),
        }
    }
}
