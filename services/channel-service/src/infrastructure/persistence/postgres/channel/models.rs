use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::channel::{Channel, ChannelError, ChannelName, ChannelType};

/// Database model for the channels table
#[derive(Debug, Clone, FromRow)]
pub struct ChannelRow {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    /// Stored as TEXT via `type::TEXT` cast in SELECT
    pub r#type: String,
    pub description: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TryFrom<ChannelRow> for Channel {
    type Error = ChannelError;

    fn try_from(row: ChannelRow) -> Result<Self, Self::Error> {
        let name = ChannelName::new(&row.name)?;
        let channel_type = row
            .r#type
            .parse::<ChannelType>()
            .map_err(|_| ChannelError::InvalidChannelType(row.r#type.clone()))?;

        Ok(Self::from_persisted(
            row.id,
            row.guild_id,
            row.parent_id,
            name,
            channel_type,
            row.description,
            row.position,
            row.created_at,
            row.updated_at,
            row.deleted_at,
        ))
    }
}

impl From<&Channel> for ChannelRow {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id(),
            guild_id: channel.guild_id(),
            parent_id: channel.parent_id(),
            name: channel.name().as_str().to_string(),
            r#type: channel.channel_type().as_str().to_string(),
            description: channel.description().map(String::from),
            position: channel.position(),
            created_at: channel.created_at(),
            updated_at: channel.updated_at(),
            deleted_at: if channel.is_deleted() {
                Some(channel.updated_at())
            } else {
                None
            },
        }
    }
}
