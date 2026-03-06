use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::guild::Guild;
use crate::domain::guild_invite::GuildInvite;
use crate::domain::guild_member::GuildMember;
use crate::domain::guild_role::GuildRole;

// ─────────────────────────────────────────────────────────────────────────────
// Write-only sub-repository traits (transactional context only)
// ─────────────────────────────────────────────────────────────────────────────

/// Transactional write operations on the `guilds` and `guild_roles` tables.
#[async_trait]
pub trait GuildWriter: Send + Sync {
    async fn save(&self, guild: &Guild) -> Result<(), RepositoryError>;
    async fn save_role(&self, role: &GuildRole) -> Result<(), RepositoryError>;
    async fn increment_member_count(&self, guild_id: Uuid) -> Result<(), RepositoryError>;
    async fn decrement_member_count(&self, guild_id: Uuid) -> Result<(), RepositoryError>;
}

/// Transactional write operations on the `guild_members` table.
#[async_trait]
pub trait GuildMemberWriter: Send + Sync {
    async fn save(&self, member: &GuildMember) -> Result<(), RepositoryError>;
    async fn update(&self, member: &GuildMember) -> Result<(), RepositoryError>;
}

/// Transactional write operations on the `guild_invites` table.
#[async_trait]
pub trait GuildInviteWriter: Send + Sync {
    async fn update(&self, invite: &GuildInvite) -> Result<(), RepositoryError>;
}

