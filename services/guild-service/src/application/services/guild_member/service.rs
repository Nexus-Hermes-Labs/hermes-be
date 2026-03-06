use std::sync::Arc;
use uuid::Uuid;

use crate::domain::guild::GuildRepository;
use crate::domain::guild_member::{GuildMember, GuildMemberRepository};
use crate::application::ports::unit_of_work::GuildUnitOfWorkFactory;

use super::error::GuildMemberServiceError;

/// Guild Member Application Service
///
/// Handles membership lifecycle: joining, leaving, kicking, and role assignment.
pub struct GuildMemberService {
    guild_repo: Arc<dyn GuildRepository>,
    member_repo: Arc<dyn GuildMemberRepository>,
    uow_factory: Arc<dyn GuildUnitOfWorkFactory>,
}

impl std::fmt::Debug for GuildMemberService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuildMemberService").finish_non_exhaustive()
    }
}

impl GuildMemberService {
    /// Create a new `GuildMemberService` with the required repository dependencies.
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
    // MEMBERSHIP
    // ============================================

    /// Add a user to a guild (called internally when using an invite)
    pub async fn join_guild(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<GuildMember, GuildMemberServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        if guild.is_full() {
            return Err(GuildMemberServiceError::GuildFull);
        }

        let already_member = self
            .member_repo
            .is_member(guild_id, user_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        if already_member {
            return Err(GuildMemberServiceError::AlreadyMember);
        }

        let member = GuildMember::new(guild_id, user_id);

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.members()
            .save(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.guilds()
            .increment_member_count(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        Ok(member)
    }

    /// Get member info
    pub async fn get_member(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<GuildMember, GuildMemberServiceError> {
        self.member_repo
            .find_by_user(guild_id, user_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::MemberNotFound)
    }

    /// Check membership
    pub async fn is_member(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, GuildMemberServiceError> {
        self.member_repo
            .is_member(guild_id, user_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))
    }

    /// List guild members (paginated)
    pub async fn list_members(
        &self,
        guild_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GuildMember>, GuildMemberServiceError> {
        // Verify guild exists
        self.guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        self.member_repo
            .find_by_guild(guild_id, limit, offset)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))
    }

    /// Kick a member (requires `KICK_MEMBERS` permission, checked by caller)
    pub async fn kick_member(
        &self,
        guild_id: Uuid,
        target_user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), GuildMemberServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        // Cannot kick the owner
        if guild.is_owner(target_user_id) {
            return Err(GuildMemberServiceError::Forbidden(
                "Cannot kick the guild owner".to_string(),
            ));
        }

        // Kicker must be a member with elevated role (simplified check)
        let requester_is_owner = guild.is_owner(requester_id);
        if !requester_is_owner {
            // TODO: Check KICK_MEMBERS permission via role service
            return Err(GuildMemberServiceError::Forbidden(
                "Insufficient permissions to kick members".to_string(),
            ));
        }

        let mut member = self.get_member(guild_id, target_user_id).await?;
        member.leave();

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.members()
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.guilds()
            .decrement_member_count(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    /// Leave a guild (voluntary)
    pub async fn leave_guild(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), GuildMemberServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        if guild.is_owner(user_id) {
            return Err(GuildMemberServiceError::Forbidden(
                "Guild owner cannot leave; transfer ownership or delete the guild first"
                    .to_string(),
            ));
        }

        let mut member = self.get_member(guild_id, user_id).await?;
        member.leave();

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.members()
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.guilds()
            .decrement_member_count(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    /// Get all guild IDs the user belongs to
    pub async fn get_user_guilds(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, GuildMemberServiceError> {
        self.member_repo
            .find_guilds_for_user(user_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))
    }

    // ============================================
    // ROLE ASSIGNMENT
    // ============================================

    /// Assign a role to a member (owner or `MANAGE_ROLES` permission required)
    pub async fn assign_role(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        requester_id: Uuid,
    ) -> Result<GuildMember, GuildMemberServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            // TODO: Check MANAGE_ROLES permission
            return Err(GuildMemberServiceError::Forbidden(
                "Insufficient permissions to manage roles".to_string(),
            ));
        }

        // Domain validation: member must exist, role must not already be assigned
        let mut member = self.get_member(guild_id, user_id).await?;
        member
            .assign_role(role_id)
            .map_err(GuildMemberServiceError::MemberDomainError)?;

        // Targeted single-row insert into the junction table
        self.member_repo
            .assign_role_to_member(guild_id, user_id, role_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        // Reload to return the fresh state from DB
        self.get_member(guild_id, user_id).await
    }

    /// Remove a role from a member
    pub async fn remove_role(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        requester_id: Uuid,
    ) -> Result<GuildMember, GuildMemberServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildMemberServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            // TODO: Check MANAGE_ROLES permission
            return Err(GuildMemberServiceError::Forbidden(
                "Insufficient permissions to manage roles".to_string(),
            ));
        }

        // Domain validation: member must exist, role must currently be held
        let mut member = self.get_member(guild_id, user_id).await?;
        member
            .remove_role(role_id)
            .map_err(GuildMemberServiceError::MemberDomainError)?;

        // Targeted single-row delete from the junction table
        self.member_repo
            .remove_role_from_member(guild_id, user_id, role_id)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        // Reload to return the fresh state from DB
        self.get_member(guild_id, user_id).await
    }
}
