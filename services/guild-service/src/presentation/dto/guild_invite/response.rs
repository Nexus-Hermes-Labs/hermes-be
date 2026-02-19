use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildInvite;

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub code: String,
    pub guild_id: Uuid,
    pub creator_id: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<GuildInvite> for InviteResponse {
    fn from(inv: GuildInvite) -> Self {
        Self {
            code: inv.code().as_str().to_string(),
            guild_id: inv.guild_id(),
            creator_id: inv.creator_id(),
            max_uses: inv.max_uses(),
            uses: inv.uses(),
            expires_at: inv.expires_at(),
            created_at: inv.created_at(),
        }
    }
}
