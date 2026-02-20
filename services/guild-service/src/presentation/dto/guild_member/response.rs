use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildMember;

/// API response representing a single guild member.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildMemberResponse {
    /// ID of the guild this membership belongs to.
    pub guild_id: Uuid,
    /// ID of the user.
    pub user_id: Uuid,
    /// Server-specific display name override (overrides the user's global username when set).
    pub nickname: Option<String>,
    /// IDs of roles currently assigned to this member.
    pub role_ids: Vec<Uuid>,
    /// Timestamp when the user joined the guild.
    pub joined_at: DateTime<Utc>,
}

impl From<GuildMember> for GuildMemberResponse {
    fn from(m: GuildMember) -> Self {
        Self {
            guild_id: m.guild_id(),
            user_id: m.user_id(),
            nickname: m.nickname().map(|n| n.as_str().to_string()),
            role_ids: m.role_ids().to_vec(),
            joined_at: m.joined_at(),
        }
    }
}

/// Paginated list of guild members.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildMemberListResponse {
    /// Members in the current page.
    pub members: Vec<GuildMemberResponse>,
    /// Total number of active members in the guild.
    pub total: i64,
}
