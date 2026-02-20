use std::sync::Arc;
use uuid::Uuid;

use crate::domain::guild::GuildRepository;
use crate::domain::guild_invite::{GuildInvite, GuildInviteRepository, InviteCode};
use crate::domain::guild_member::{GuildMember, GuildMemberRepository};

use super::error::GuildInviteServiceError;

/// Guild Invite Application Service
///
/// Handles the full invite lifecycle: creation, lookup, redemption, and revocation.
#[allow(clippy::struct_field_names)]
pub struct GuildInviteService {
    guild_repo: Arc<dyn GuildRepository>,
    invite_repo: Arc<dyn GuildInviteRepository>,
    member_repo: Arc<dyn GuildMemberRepository>,
}

impl std::fmt::Debug for GuildInviteService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuildInviteService").finish_non_exhaustive()
    }
}

impl GuildInviteService {
    /// Create a new `GuildInviteService` with the required repository dependencies.
    pub fn new(
        guild_repo: Arc<dyn GuildRepository>,
        invite_repo: Arc<dyn GuildInviteRepository>,
        member_repo: Arc<dyn GuildMemberRepository>,
    ) -> Self {
        Self {
            guild_repo,
            invite_repo,
            member_repo,
        }
    }

    // ============================================
    // INVITE MANAGEMENT
    // ============================================

    /// Create an invite (caller must be a guild member)
    pub async fn create_invite(
        &self,
        guild_id: Uuid,
        creator_id: Uuid,
        max_uses: Option<i32>,
        max_age_seconds: Option<i64>,
    ) -> Result<GuildInvite, GuildInviteServiceError> {
        // Verify guild exists
        self.guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::GuildNotFound)?;

        // Caller must be a member
        let is_member = self
            .member_repo
            .is_member(guild_id, creator_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        if !is_member {
            return Err(GuildInviteServiceError::Forbidden(
                "Only guild members can create invites".to_string(),
            ));
        }

        let invite = GuildInvite::new(guild_id, creator_id, max_uses, max_age_seconds);

        self.invite_repo
            .save(&invite)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        Ok(invite)
    }

    /// Get invite info by code
    pub async fn get_invite(&self, code: String) -> Result<GuildInvite, GuildInviteServiceError> {
        let invite_code = InviteCode::from_existing(code);
        self.invite_repo
            .find_by_code(&invite_code)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::InviteNotFound)
    }

    /// Use an invite to join a guild
    ///
    /// Returns the new `GuildMember` record on success.
    pub async fn use_invite(
        &self,
        code: String,
        user_id: Uuid,
    ) -> Result<GuildMember, GuildInviteServiceError> {
        let invite_code = InviteCode::from_existing(&code);

        let mut invite = self
            .invite_repo
            .find_by_code(&invite_code)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::InviteNotFound)?;

        if !invite.is_valid() {
            return Err(GuildInviteServiceError::InviteInvalid);
        }

        let guild_id = invite.guild_id();

        // Check if already member
        let already_member = self
            .member_repo
            .is_member(guild_id, user_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        if already_member {
            return Err(GuildInviteServiceError::AlreadyMember);
        }

        // Check guild capacity
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::GuildNotFound)?;

        if guild.is_full() {
            return Err(GuildInviteServiceError::GuildFull);
        }

        // Consume one use
        invite
            .use_invite()
            .map_err(GuildInviteServiceError::DomainError)?;

        self.invite_repo
            .update(&invite)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        // Create member record
        let member = GuildMember::new(guild_id, user_id);

        self.member_repo
            .save(&member)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        self.guild_repo
            .increment_member_count(guild_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        Ok(member)
    }

    /// Revoke an invite
    pub async fn revoke_invite(
        &self,
        code: String,
        requester_id: Uuid,
    ) -> Result<(), GuildInviteServiceError> {
        let invite_code = InviteCode::from_existing(&code);

        let mut invite = self
            .invite_repo
            .find_by_code(&invite_code)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::InviteNotFound)?;

        let guild = self
            .guild_repo
            .find_by_id(invite.guild_id())
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::GuildNotFound)?;

        // Guild owner can revoke any invite; creator can revoke their own
        if guild.is_owner(requester_id) {
            // Owner: force revoke by treating them as the creator
            invite
                .revoke(invite.creator_id())
                .map_err(GuildInviteServiceError::DomainError)?;
        } else {
            invite
                .revoke(requester_id)
                .map_err(GuildInviteServiceError::DomainError)?;
        }

        self.invite_repo
            .update(&invite)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    /// List active invites for a guild
    pub async fn list_invites(
        &self,
        guild_id: Uuid,
        requester_id: Uuid,
    ) -> Result<Vec<GuildInvite>, GuildInviteServiceError> {
        let guild = self
            .guild_repo
            .find_by_id(guild_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))?
            .ok_or(GuildInviteServiceError::GuildNotFound)?;

        if !guild.is_owner(requester_id) {
            // TODO: Check MANAGE_GUILD permission
            return Err(GuildInviteServiceError::Forbidden(
                "Only guild admins can view all invites".to_string(),
            ));
        }

        self.invite_repo
            .find_by_guild(guild_id)
            .await
            .map_err(|e| GuildInviteServiceError::RepositoryError(e.to_string()))
    }
}
