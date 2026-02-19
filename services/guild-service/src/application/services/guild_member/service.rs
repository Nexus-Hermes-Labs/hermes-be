use std::sync::Arc;
use uuid::Uuid;

use crate::domain::guild::GuildRepository;
use crate::domain::guild_member::{GuildMember, GuildMemberRepository};

use super::error::GuildMemberServiceError;

/// Guild Member Application Service
pub struct GuildMemberService {
    guild_repo: Arc<dyn GuildRepository>,
    member_repo: Arc<dyn GuildMemberRepository>,
}

impl GuildMemberService {
    pub fn new(
        guild_repo: Arc<dyn GuildRepository>,
        member_repo: Arc<dyn GuildMemberRepository>,
    ) -> Self {
        Self {
            guild_repo,
            member_repo,
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

        self.member_repo
            .save(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        self.guild_repo
            .increment_member_count(guild_id)
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

    /// Kick a member (requires KICK_MEMBERS permission, checked by caller)
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

        self.member_repo
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        self.guild_repo
            .decrement_member_count(guild_id)
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
                "Guild owner cannot leave; transfer ownership or delete the guild first".to_string(),
            ));
        }

        let mut member = self.get_member(guild_id, user_id).await?;
        member.leave();

        self.member_repo
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        self.guild_repo
            .decrement_member_count(guild_id)
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

    /// Assign a role to a member (owner or MANAGE_ROLES permission required)
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

        let mut member = self.get_member(guild_id, user_id).await?;
        member.assign_role(role_id).map_err(GuildMemberServiceError::MemberDomainError)?;

        self.member_repo
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        Ok(member)
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

        let mut member = self.get_member(guild_id, user_id).await?;
        member.remove_role(role_id).map_err(GuildMemberServiceError::MemberDomainError)?;

        self.member_repo
            .update(&member)
            .await
            .map_err(|e| GuildMemberServiceError::RepositoryError(e.to_string()))?;

        Ok(member)
    }
}
