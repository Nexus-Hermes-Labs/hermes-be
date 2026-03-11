use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::unit_of_work::ChannelUnitOfWorkFactory;
use crate::domain::channel::{Channel, ChannelName, ChannelRepository, ChannelType};
use crate::infrastructure::grpc::guild_client::GuildGrpcClient;

use super::error::ChannelServiceError;

/// Channel Application Service
///
/// Orchestrates channel CRUD operations, delegating persistence to the
/// repository and guild authorization to the guild gRPC client.
pub struct ChannelService {
    channel_repo: Arc<dyn ChannelRepository>,
    uow_factory: Arc<dyn ChannelUnitOfWorkFactory>,
    guild_client: Arc<GuildGrpcClient>,
}

impl std::fmt::Debug for ChannelService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelService").finish_non_exhaustive()
    }
}

impl ChannelService {
    /// Create a new `ChannelService` with required dependencies.
    pub fn new(
        channel_repo: Arc<dyn ChannelRepository>,
        uow_factory: Arc<dyn ChannelUnitOfWorkFactory>,
        guild_client: Arc<GuildGrpcClient>,
    ) -> Self {
        Self {
            channel_repo,
            uow_factory,
            guild_client,
        }
    }

    // ============================================
    // CHANNEL MANAGEMENT
    // ============================================

    /// Create a new channel in a guild.
    ///
    /// Verifies that the guild exists and that `requester_id` is the guild owner.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_channel(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
        name: String,
        channel_type: String,
        parent_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<Channel, ChannelServiceError> {
        // Verify guild exists and check ownership
        let guild = self
            .guild_client
            .get_guild(guild_id)
            .await
            .map_err(|e| ChannelServiceError::GrpcError(e.to_string()))?
            .ok_or(ChannelServiceError::GuildNotFound)?;

        let owner_id: Uuid = guild.owner_id.parse().map_err(|_| {
            ChannelServiceError::GrpcError("Invalid owner_id from guild".to_string())
        })?;

        if owner_id != requester_id {
            return Err(ChannelServiceError::Forbidden(
                "Only the guild owner can create channels".to_string(),
            ));
        }

        let channel_name = ChannelName::new(&name).map_err(ChannelServiceError::DomainError)?;
        let channel_type_vo = channel_type
            .parse::<ChannelType>()
            .map_err(ChannelServiceError::DomainError)?;

        // Assign position = max existing position + 1
        let position = self
            .channel_repo
            .max_position(guild_id)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?
            + 1;

        let channel = Channel::new(
            guild_id,
            parent_id,
            channel_name,
            channel_type_vo,
            description,
            position,
        )
        .map_err(ChannelServiceError::DomainError)?;

        self.channel_repo
            .save(&channel)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        Ok(channel)
    }

    /// Create default text + voice channels for a newly created guild.
    ///
    /// Skips ownership check — this is an internal service-to-service call
    /// invoked immediately after guild creation when the guild is brand new.
    pub async fn create_default_channels(
        &self,
        guild_id: Uuid,
    ) -> Result<(Channel, Channel), ChannelServiceError> {
        let text_name = ChannelName::new("general").map_err(ChannelServiceError::DomainError)?;
        let voice_name = ChannelName::new("General").map_err(ChannelServiceError::DomainError)?;

        let text_channel = Channel::new(guild_id, None, text_name, ChannelType::Text, None, 1)
            .map_err(ChannelServiceError::DomainError)?;

        let voice_channel = Channel::new(guild_id, None, voice_name, ChannelType::Voice, None, 2)
            .map_err(ChannelServiceError::DomainError)?;

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        uow.channels()
            .save(&text_channel)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        uow.channels()
            .save(&voice_channel)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        Ok((text_channel, voice_channel))
    }

    /// Get a channel by ID.
    pub async fn get_channel(&self, channel_id: Uuid) -> Result<Channel, ChannelServiceError> {
        self.channel_repo
            .find_by_id(channel_id)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?
            .ok_or(ChannelServiceError::ChannelNotFound)
    }

    /// List all channels in a guild, ordered by position.
    pub async fn list_guild_channels(
        &self,
        guild_id: Uuid,
    ) -> Result<Vec<Channel>, ChannelServiceError> {
        self.channel_repo
            .find_by_guild(guild_id)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))
    }

    /// Update a channel's settings.
    ///
    /// Verifies that `requester_id` is the guild owner before applying changes.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_channel(
        &self,
        channel_id: Uuid,
        requester_id: Uuid,
        name: Option<String>,
        parent_id: Option<Uuid>,
        description: Option<String>,
        position: Option<i32>,
    ) -> Result<Channel, ChannelServiceError> {
        let mut channel = self.get_channel(channel_id).await?;

        // Verify ownership via guild gRPC
        let guild = self
            .guild_client
            .get_guild(channel.guild_id())
            .await
            .map_err(|e| ChannelServiceError::GrpcError(e.to_string()))?
            .ok_or(ChannelServiceError::GuildNotFound)?;

        let owner_id: Uuid = guild.owner_id.parse().map_err(|_| {
            ChannelServiceError::GrpcError("Invalid owner_id from guild".to_string())
        })?;

        if owner_id != requester_id {
            return Err(ChannelServiceError::Forbidden(
                "Only the guild owner can update channels".to_string(),
            ));
        }

        let name_vo = name
            .map(|n| ChannelName::new(n).map_err(ChannelServiceError::DomainError))
            .transpose()?;

        channel
            .update(name_vo, parent_id, description, position)
            .map_err(ChannelServiceError::DomainError)?;

        self.channel_repo
            .update(&channel)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))?;

        Ok(channel)
    }

    /// Delete a channel (soft delete).
    ///
    /// Verifies that `requester_id` is the guild owner before deleting.
    pub async fn delete_channel(
        &self,
        channel_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), ChannelServiceError> {
        let channel = self.get_channel(channel_id).await?;

        // Verify ownership via guild gRPC
        let guild = self
            .guild_client
            .get_guild(channel.guild_id())
            .await
            .map_err(|e| ChannelServiceError::GrpcError(e.to_string()))?
            .ok_or(ChannelServiceError::GuildNotFound)?;

        let owner_id: Uuid = guild.owner_id.parse().map_err(|_| {
            ChannelServiceError::GrpcError("Invalid owner_id from guild".to_string())
        })?;

        if owner_id != requester_id {
            return Err(ChannelServiceError::Forbidden(
                "Only the guild owner can delete channels".to_string(),
            ));
        }

        self.channel_repo
            .delete(channel_id)
            .await
            .map_err(|e| ChannelServiceError::RepositoryError(e.to_string()))
    }
}
