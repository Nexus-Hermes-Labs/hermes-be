use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Guild;

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildResponse {
    pub guild_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub visibility: String,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildListResponse {
    pub guilds: Vec<GuildResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
