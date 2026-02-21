use uuid::Uuid;

/// Domain events emitted by the channel aggregate (for NATS publication)
#[derive(Debug, Clone)]
pub enum ChannelEvent {
    /// A new channel was created in a guild
    ChannelCreated {
        channel_id: Uuid,
        guild_id: Uuid,
        channel_type: String,
    },
    /// Channel settings were updated
    ChannelUpdated { channel_id: Uuid, guild_id: Uuid },
    /// A channel was deleted
    ChannelDeleted { channel_id: Uuid, guild_id: Uuid },
}
