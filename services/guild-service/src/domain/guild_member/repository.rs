use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use super::entity::GuildMember;

/// Guild member repository trait
///
/// Keyed by composite (guild_id, user_id) rather than a single Uuid,
/// so we do not implement the generic Repository<GuildMember, Uuid> here.
#[async_trait]
pub trait GuildMemberRepository: Send + Sync {
    // ============================================
    // CRUD
    // ============================================

    async fn save(&self, member: &GuildMember) -> Result<(), RepositoryError>;
    async fn update(&self, member: &GuildMember) -> Result<(), RepositoryError>;

    /// Find active member by (guild_id, user_id)
    async fn find_by_user(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GuildMember>, RepositoryError>;

    /// Check membership
    async fn is_member(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // ============================================
    // LISTING
    // ============================================

    /// List all active members of a guild (paginated)
    async fn find_by_guild(
        &self,
        guild_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GuildMember>, RepositoryError>;

    /// Count active members of a guild
    async fn count_by_guild(
        &self,
        guild_id: Uuid,
    ) -> Result<i64, RepositoryError>;

    /// Get all guilds a user belongs to (active memberships)
    async fn find_guilds_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, RepositoryError>;

    // ============================================
    // ROLE QUERIES
    // ============================================

    /// Get all members who have a specific role
    async fn find_by_role(
        &self,
        guild_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<GuildMember>, RepositoryError>;
}
