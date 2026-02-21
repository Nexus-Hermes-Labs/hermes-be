use uuid::Uuid;

/// Events related to channels
#[derive(Debug, Clone)]
pub enum ChannelEvent {
    /// A new channel was created in a guild
    Created {
        channel_id: Uuid,
        guild_id: Uuid,
        name: String,
    },
    /// A channel's metadata was updated
    Updated { channel_id: Uuid, guild_id: Uuid },
    /// A channel was deleted
    Deleted { channel_id: Uuid, guild_id: Uuid },
}
