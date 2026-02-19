use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::guild_member::{GuildMember, GuildMemberError, MemberNickname};

/// Database model for guild_members table
#[derive(Debug, Clone, FromRow)]
pub struct GuildMemberRow {
    pub guild_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    /// Stored as a comma-separated UUID list or JSON array in Postgres
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
            .map(|n| MemberNickname::new(n))
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

impl From<&GuildMember> for GuildMemberRow {
    fn from(member: &GuildMember) -> Self {
        Self {
            guild_id: member.guild_id(),
            user_id: member.user_id(),
            nickname: member.nickname().map(|n| n.as_str().to_string()),
            role_ids: member.role_ids().to_vec(),
            joined_at: member.joined_at(),
            updated_at: member.updated_at(),
            left_at: if member.is_active() { None } else { Some(member.updated_at()) },
        }
    }
}
