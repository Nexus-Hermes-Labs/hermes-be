use crate::application::{GuildInviteService, GuildMemberService, GuildRoleService, GuildService};
use std::sync::Arc;

/// Guild domain state
///
/// Contains pre-composed guild services.
#[derive(Clone)]
pub struct GuildState {
    pub guild_service: Arc<GuildService>,
    pub member_service: Arc<GuildMemberService>,
    pub role_service: Arc<GuildRoleService>,
    pub invite_service: Arc<GuildInviteService>,
}

impl GuildState {
    pub fn new(
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
