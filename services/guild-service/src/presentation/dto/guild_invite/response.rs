use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::GuildInvite;

/// API response representing a guild invite.
#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    /// 8-character alphanumeric invite code (case-sensitive).
    pub code: String,
    /// ID of the guild this invite grants access to.
    pub guild_id: Uuid,
    /// ID of the member who created the invite.
    pub creator_id: Uuid,
    /// Maximum number of allowed uses (`None` = unlimited).
    pub max_uses: Option<i32>,
    /// Number of times this invite has been used so far.
    pub uses: i32,
    /// Timestamp after which the invite is no longer valid (`None` = never expires).
    pub expires_at: Option<DateTime<Utc>>,
    /// Timestamp when the invite was created.
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
