use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::guild::{Guild, GuildError, GuildName, GuildVisibility};

/// Database model for guilds table
#[derive(Debug, Clone, FromRow)]
pub struct GuildRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub visibility: String,
    pub member_count: i32,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TryFrom<GuildRow> for Guild {
    type Error = GuildError;

    fn try_from(row: GuildRow) -> Result<Self, Self::Error> {
        let name = GuildName::new(&row.name)?;
        let visibility = row
            .visibility
            .parse::<GuildVisibility>()
            .map_err(|_| GuildError::InvalidNameFormat)?; // Reuse error as a fallback

        Ok(Self::from_persisted(
            row.id,
            row.owner_id,
            name,
            row.description,
            row.icon_url,
            row.banner_url,
            visibility,
            row.member_count,
            row.max_members,
            row.created_at,
            row.updated_at,
            row.deleted_at,
        ))
    }
}

impl From<&Guild> for GuildRow {
    fn from(guild: &Guild) -> Self {
        Self {
            id: guild.id(),
            owner_id: guild.owner_id(),
            name: guild.name().as_str().to_string(),
            description: guild.description().map(String::from),
            icon_url: guild.icon_url().map(String::from),
            banner_url: guild.banner_url().map(String::from),
            visibility: guild.visibility().as_str().to_string(),
            member_count: guild.member_count(),
            max_members: guild.max_members(),
            created_at: guild.created_at(),
            updated_at: guild.updated_at(),
            deleted_at: if guild.is_deleted() {
                Some(guild.updated_at())
            } else {
                None
            },
        }
    }
}
