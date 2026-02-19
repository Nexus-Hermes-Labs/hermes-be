use std::sync::Arc;
use uuid::Uuid;

use crate::domain::guild::GuildRepository;
use crate::domain::guild_role::{GuildRole, GuildRoleRepository, Permissions, RoleColor};

use super::error::GuildRoleServiceError;

/// Guild Role Application Service
pub struct GuildRoleService {
    guild_repo: Arc<dyn GuildRepository>,
    role_repo: Arc<dyn GuildRoleRepository>,
}

impl GuildRoleService {
    pub fn new(
        guild_repo: Arc<dyn GuildRepository>,
        role_repo: Arc<dyn GuildRoleRepository>,
    ) -> Self {
        Self {
            guild_repo,
            role_repo,
        }
    }

    // ============================================
    // ROLE MANAGEMENT
    // ============================================

    /// Create a new role in a guild (owner or MANAGE_ROLES required)
    pub async fn create_role(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
        name: String,
        permissions: Permissions,
    ) -> Result<GuildRole, GuildRoleServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildRoleServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            // TODO: Check MANAGE_ROLES permission
            return Err(GuildRoleServiceError::Forbidden(
                "Insufficient permissions to create roles".to_string(),
            ));
        }

        // Check name uniqueness
        if self.role_repo.exists_by_name(guild_id, &name).await? {
            return Err(GuildRoleServiceError::RoleNameTaken);
        }

        // Determine next position
        let existing = self.role_repo.find_by_guild(guild_id).await?;
        let next_position = existing.iter().map(|r| r.position()).max().unwrap_or(0) + 1;

        let role = GuildRole::new(guild_id, name, permissions, next_position)
            .map_err(GuildRoleServiceError::DomainError)?;

        self.role_repo
            .save(&role)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?;

        Ok(role)
    }

    /// Get all roles for a guild
    pub async fn list_roles(
        &self,
        guild_id: Uuid,
    ) -> Result<Vec<GuildRole>, GuildRoleServiceError> {
        self.guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildRoleServiceError::GuildNotFound)?;

        self.role_repo
            .find_by_guild(guild_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))
    }

    /// Get a single role
    pub async fn get_role(
        &self,
        role_id: Uuid,
    ) -> Result<GuildRole, GuildRoleServiceError> {
        self.role_repo
            .find_by_id(role_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildRoleServiceError::RoleNotFound)
    }

    /// Update a role
    pub async fn update_role(
        &self,
        guild_id: Uuid,
        role_id: Uuid,
        requester_id: Uuid,
        name: Option<String>,
        color: Option<RoleColor>,
        permissions: Option<Permissions>,
        hoist: Option<bool>,
        mentionable: Option<bool>,
    ) -> Result<GuildRole, GuildRoleServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildRoleServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            return Err(GuildRoleServiceError::Forbidden(
                "Insufficient permissions to update roles".to_string(),
            ));
        }

        let mut role = self.get_role(role_id).await?;

        role.update(name, color, permissions, hoist, mentionable)
            .map_err(GuildRoleServiceError::DomainError)?;

        self.role_repo
            .update(&role)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?;

        Ok(role)
    }

    /// Delete a role (cannot delete @everyone)
    pub async fn delete_role(
        &self,
        guild_id: Uuid,
        role_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), GuildRoleServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildRoleServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            return Err(GuildRoleServiceError::Forbidden(
                "Insufficient permissions to delete roles".to_string(),
            ));
        }

        let role = self.get_role(role_id).await?;

        if role.is_default() {
            return Err(GuildRoleServiceError::CannotDeleteDefaultRole);
        }

        self.role_repo
            .delete(role_id)
            .await
            .map_err(|e| GuildRoleServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}
