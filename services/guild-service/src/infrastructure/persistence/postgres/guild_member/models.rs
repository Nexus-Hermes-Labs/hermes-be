use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::guild_member::{GuildMember, GuildMemberError, MemberNickname};

/// Database read model for guild_members joined with guild_member_roles.
///
/// `role_ids` is populated by aggregating the `guild_member_roles` junction table;
/// it is NOT a stored column on `guild_members`.
#[derive(Debug, Clone, FromRow)]
pub struct GuildMemberRow {
    pub guild_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    /// Aggregated from guild_member_roles via LEFT JOIN + ARRAY_AGG
    pub role_ids: Vec<Uuid>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

impl TryFrom<GuildMemberRow> for GuildMember {
    type Error = GuildMemberError;

    fn try_from(row: GuildMemberRow) -> Result<Self, Self::Error> {
        let nickname = row
            .nickname
            .map(MemberNickname::new)
            .transpose()?;

        Ok(GuildMember::from_persisted(
            row.guild_id,
            row.user_id,
            nickname,
            row.role_ids,
            row.joined_at,
            row.updated_at,
            row.left_at,
        ))
    }
}
