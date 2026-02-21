use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Channel;

/// Channel representation returned by API responses
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChannelResponse {
    /// Unique channel identifier
    pub id: Uuid,
    /// Guild this channel belongs to
    pub guild_id: Uuid,
    /// Parent category channel, if any
    pub parent_id: Option<Uuid>,
    /// Channel display name
    pub name: String,
    /// Channel type: "text", "voice", "category", or "announcement"
    pub channel_type: String,
    /// Channel topic or description
    pub description: Option<String>,
    /// Display position within the guild (ascending order)
    pub position: i32,
    /// Timestamp when the channel was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the most recent update
    pub updated_at: DateTime<Utc>,
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id(),
            guild_id: channel.guild_id(),
            parent_id: channel.parent_id(),
            name: channel.name().as_str().to_string(),
            channel_type: channel.channel_type().as_str().to_string(),
            description: channel.description().map(String::from),
            position: channel.position(),
            created_at: channel.created_at(),
            updated_at: channel.updated_at(),
        }
    }
}

/// Paginated list of channels
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChannelListResponse {
    /// Ordered list of channels (by position)
    pub channels: Vec<ChannelResponse>,
    /// Total number of channels returned
    pub total: i64,
}
