use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ChannelType {
    Text,
    Voice,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Channel {
    pub fn new(guild_id: Uuid, name: String, channel_type: ChannelType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            guild_id,
            name,
            channel_type,
            created_at: now,
            updated_at: now,
        }
    }
}
