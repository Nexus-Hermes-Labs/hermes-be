use async_trait::async_trait;
use uuid::Uuid;

/// Minimal guild data needed by the channel application layer.
///
/// Only carries the fields actually consumed by `ChannelService`; tonic-specific
/// types stay in the infrastructure layer.
#[derive(Debug, Clone)]
pub struct GuildInfo {
    /// User ID of the guild's owner.
    pub owner_id: Uuid,
}

/// Port (outbound) for querying guild information from an external service.
///
/// Defined here in the application layer so that `ChannelService` depends only
/// on this abstraction.  The concrete implementation (`GuildGrpcClient`) lives in
/// `infrastructure/grpc/` and is injected at bootstrap time.
#[async_trait]
pub trait GuildClient: Send + Sync {
    /// Return guild metadata for `guild_id`, or `None` if the guild does not exist.
    async fn get_guild(&self, guild_id: Uuid) -> Result<Option<GuildInfo>, String>;
}
