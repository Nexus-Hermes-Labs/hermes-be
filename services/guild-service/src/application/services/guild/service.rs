use std::sync::Arc;
use uuid::Uuid;

use crate::domain::guild::{Guild, GuildName, GuildRepository, GuildVisibility};
use crate::domain::guild_member::GuildMemberRepository;
use crate::domain::guild_role::GuildRole;
use crate::application::ports::unit_of_work::GuildUnitOfWorkFactory;

use super::error::GuildServiceError;

/// Guild Application Service
///
/// Orchestrates guild creation, settings updates, and deletion.
#[allow(clippy::struct_field_names)]
pub struct GuildService {
    guild_repo: Arc<dyn GuildRepository>,
    member_repo: Arc<dyn GuildMemberRepository>,
    uow_factory: Arc<dyn GuildUnitOfWorkFactory>,
}

impl std::fmt::Debug for GuildService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuildService").finish_non_exhaustive()
    }
}

impl GuildService {
    /// Create a new `GuildService` with the required repository dependencies.
    pub fn new(
        guild_repo: Arc<dyn GuildRepository>,
        member_repo: Arc<dyn GuildMemberRepository>,
        uow_factory: Arc<dyn GuildUnitOfWorkFactory>,
    ) -> Self {
        Self {
            guild_repo,
            member_repo,
            uow_factory,
        }
    }

    // ============================================
    // GUILD MANAGEMENT
    // ============================================

    /// Create a new guild and its mandatory `@everyone` role atomically.
    pub async fn create_guild(
        &self,
        owner_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Guild, GuildServiceError> {
        let guild_name = GuildName::new(&name).map_err(GuildServiceError::DomainError)?;

        let guild = Guild::new(owner_id, guild_name, description)
            .map_err(GuildServiceError::DomainError)?;

        let everyone_role = GuildRole::new_everyone(guild.id());

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        uow.guilds()
            .save(&guild)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        uow.guilds()
            .save_role(&everyone_role)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        Ok(guild)
    }

    /// Get guild by ID
    pub async fn get_guild(&self, guild_id: Uuid) -> Result<Guild, GuildServiceError> {
        self.guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildServiceError::GuildNotFound)
    }

    /// Update guild settings
    #[allow(clippy::too_many_arguments)]
    pub async fn update_guild(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        icon_url: Option<String>,
        banner_url: Option<String>,
        visibility: Option<GuildVisibility>,
    ) -> Result<Guild, GuildServiceError> {
        let mut guild = self.get_guild(guild_id).await?;

        if !guild.is_owner(requester_id) {
            return Err(GuildServiceError::Forbidden(
                "Only the guild owner can update settings".to_string(),
            ));
        }

        let name_vo = name
            .map(|n| GuildName::new(n).map_err(GuildServiceError::DomainError))
            .transpose()?;

        guild
            .update(name_vo, description, icon_url, banner_url, visibility)
            .map_err(GuildServiceError::DomainError)?;

        self.guild_repo
            .update(&guild)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        Ok(guild)
    }

    /// Delete guild (owner only, cascades to members/roles/invites at infra level)
    pub async fn delete_guild(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), GuildServiceError> {
        let mut guild = self.get_guild(guild_id).await?;

        guild
            .delete(requester_id)
            .map_err(GuildServiceError::DomainError)?;

        self.guild_repo
            .update(&guild)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    /// Transfer guild ownership
    pub async fn transfer_ownership(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
        new_owner_id: Uuid,
    ) -> Result<Guild, GuildServiceError> {
        // Verify new owner is actually a member
        let is_member = self
            .member_repo
            .is_member(guild_id, new_owner_id)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        if !is_member {
            return Err(GuildServiceError::Forbidden(
                "New owner must be a member of the guild".to_string(),
            ));
        }

        let mut guild = self.get_guild(guild_id).await?;

        guild
            .transfer_ownership(requester_id, new_owner_id)
            .map_err(GuildServiceError::DomainError)?;

        self.guild_repo
            .update(&guild)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))?;

        Ok(guild)
    }

    /// Get guilds owned by a user
    pub async fn get_owned_guilds(&self, owner_id: Uuid) -> Result<Vec<Guild>, GuildServiceError> {
        self.guild_repo
            .find_by_owner(owner_id)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))
    }

    /// Search public guilds
    pub async fn search_guilds(
        &self,
        query: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Guild>, GuildServiceError> {
        self.guild_repo
            .search_public(&query, limit, offset)
            .await
            .map_err(|e| GuildServiceError::RepositoryError(e.to_string()))
    }
}
