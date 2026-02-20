use crate::application::{GuildInviteService, GuildMemberService, GuildRoleService, GuildService};
use std::sync::Arc;

/// Guild domain state
///
/// Holds all pre-composed guild application services, shared across handlers via `AppState`.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug)]
pub struct GuildState {
    /// Handles guild CRUD and ownership management.
    pub guild_service: Arc<GuildService>,
    /// Handles membership lifecycle (join, leave, kick, role assignment).
    pub member_service: Arc<GuildMemberService>,
    /// Manages the role hierarchy within guilds.
    pub role_service: Arc<GuildRoleService>,
    /// Manages invite creation, redemption, and revocation.
    pub invite_service: Arc<GuildInviteService>,
}

impl GuildState {
    /// Compose a `GuildState` from pre-built application services.
    #[must_use]
    pub const fn new(
        guild_service: Arc<GuildService>,
        member_service: Arc<GuildMemberService>,
        role_service: Arc<GuildRoleService>,
        invite_service: Arc<GuildInviteService>,
    ) -> Self {
        Self {
            guild_service,
            member_service,
            role_service,
            invite_service,
        }
    }
}
