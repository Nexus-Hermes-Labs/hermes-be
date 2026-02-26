use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Guild;

/// API response representing a single guild.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildResponse {
    /// Unique guild identifier.
    pub guild_id: Uuid,
    /// User ID of the guild owner.
    pub owner_id: Uuid,
    /// Guild display name.
    pub name: String,
    /// Optional guild description.
    pub description: Option<String>,
    /// URL of the guild icon image.
    pub icon_url: Option<String>,
    /// URL of the guild banner image.
    pub banner_url: Option<String>,
    /// Visibility setting: `"public"` or `"private"`.
    pub visibility: String,
    /// Cached count of active members.
    pub member_count: i32,
    /// Timestamp when the guild was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the most recent guild update.
    pub updated_at: DateTime<Utc>,
}

impl From<Guild> for GuildResponse {
    fn from(g: Guild) -> Self {
        Self {
            guild_id: g.id(),
            owner_id: g.owner_id(),
            name: g.name().as_str().to_string(),
            description: g.description().map(String::from),
            icon_url: g.icon_url().map(String::from),
            banner_url: g.banner_url().map(String::from),
            visibility: g.visibility().as_str().to_string(),
            member_count: g.member_count(),
            created_at: g.created_at(),
            updated_at: g.updated_at(),
        }
    }
}

/// Response returned when a guild is first created, including the ID of the
/// default text channel so the client can navigate directly into the guild.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateGuildResponse {
    #[serde(flatten)]
    pub guild: GuildResponse,
    /// ID of the default `#general` text channel created with the guild.
    pub first_channel_id: Uuid,
}

/// Paginated list of guilds returned from a search.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildListResponse {
    /// Guilds in the current page.
    pub guilds: Vec<GuildResponse>,
    /// Total number of results matching the query (before pagination).
    pub total: i64,
    /// Maximum number of results requested.
    pub limit: i64,
    /// Number of results skipped.
    pub offset: i64,
}
