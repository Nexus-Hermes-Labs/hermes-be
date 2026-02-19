use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use super::entity::GuildInvite;
use super::valueobject::InviteCode;

/// Guild invite repository trait
#[async_trait]
pub trait GuildInviteRepository: Send + Sync {
    async fn save(&self, invite: &GuildInvite) -> Result<(), RepositoryError>;
    async fn update(&self, invite: &GuildInvite) -> Result<(), RepositoryError>;

    /// Find invite by code (case-sensitive)
    async fn find_by_code(
        &self,
        code: &InviteCode,
    ) -> Result<Option<GuildInvite>, RepositoryError>;

    /// Get all active invites for a guild
    async fn find_by_guild(
        &self,
        guild_id: Uuid,
    ) -> Result<Vec<GuildInvite>, RepositoryError>;

    /// Get all invites created by a user in a guild
    async fn find_by_creator(
        &self,
        guild_id: Uuid,
        creator_id: Uuid,
    ) -> Result<Vec<GuildInvite>, RepositoryError>;

    /// Delete all invites for a guild (on guild deletion)
    async fn delete_by_guild(
        &self,
        guild_id: Uuid,
    ) -> Result<(), RepositoryError>;
}
