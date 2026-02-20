#![allow(clippy::needless_for_each)]
use utoipa::OpenApi;

#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild::response::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_invite::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_invite::response::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_member::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_member::response::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_role::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::guild_role::response::*;

#[allow(clippy::wildcard_imports)]
use crate::presentation::http::handlers::{
    guild::*, guild_invite::*, guild_member::*, guild_role::*,
};

/// `OpenAPI` documentation for the guild service.
#[allow(clippy::needless_for_each)]
#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        // Guild
        create_guild,
        get_guild,
        update_guild,
        delete_guild,
        search_guilds,
        // Members
        list_members,
        get_member,
        kick_member,
        leave_guild,
        assign_role,
        remove_role,
        // Roles
        create_role,
        list_roles,
        get_role,
        update_role,
        delete_role,
        // Invites
        create_invite,
        get_invite,
        use_invite,
        revoke_invite,
        list_invites,
    ),
    components(schemas(
        CreateGuildRequest,
        UpdateGuildRequest,
        GuildResponse,
        GuildListResponse,
        KickMemberRequest,
        AssignRoleRequest,
        GuildMemberResponse,
        GuildMemberListResponse,
        CreateRoleRequest,
        UpdateRoleRequest,
        GuildRoleResponse,
        CreateInviteRequest,
        InviteResponse,
    )),
    tags(
        (name = "guilds", description = "Guild management"),
        (name = "guild-members", description = "Guild membership"),
        (name = "guild-roles", description = "Role management"),
        (name = "guild-invites", description = "Invite management"),
    ),
    info(
        title = "Guild Service API",
        version = "0.1.0",
        description = "Handles guilds, roles, members, invites, and permissions"
    )
)]
pub struct ApiDoc;
