use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildMember;

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildMemberResponse {
    pub guild_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    pub role_ids: Vec<Uuid>,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildMemberListResponse {
    pub members: Vec<GuildMemberResponse>,
    pub total: i64,
}
