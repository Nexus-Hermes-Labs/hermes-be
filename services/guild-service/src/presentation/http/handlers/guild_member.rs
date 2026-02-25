use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::presentation::dto::guild_member::request::{
    AssignRoleRequest, KickMemberRequest, Pagination,
};
use crate::presentation::dto::guild_member::response::{
    GuildMemberListResponse, GuildMemberResponse,
};
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// GET `/api/v1/guilds/:guild_id/members`
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/members",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "Members listed", body = GuildMemberListResponse),
        (status = 400, description = "Invalid Guild ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Guild not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guild-members"
)]
pub async fn list_members(
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<GuildMemberListResponse>, ApiError> {
    pagination.validate()?;
    let members = state
        .guild
        .member_service
        .list_members(guild_id, pagination.limit, pagination.offset)
        .await?;
    let total = i64::try_from(members.len()).unwrap_or(i64::MAX);
    Ok(Json(GuildMemberListResponse {
        members: members.into_iter().map(GuildMemberResponse::from).collect(),
        total,
    }))
}

/// GET `/api/v1/guilds/:guild_id/members/:user_id`
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/members/{user_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member found", body = GuildMemberResponse),
        (status = 400, description = "Invalid Guild ID or User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Member not found"),
    ),
    tag = "guild-members"
)]
pub async fn get_member(
    State(state): State<AppState>,
    Path((guild_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GuildMemberResponse>, ApiError> {
    let member = state
        .guild
        .member_service
        .get_member(guild_id, user_id)
        .await?;
    Ok(Json(GuildMemberResponse::from(member)))
}

/// DELETE `/api/v1/guilds/:guild_id/members/:user_id`
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}/members/{user_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("user_id" = Uuid, Path, description = "User ID to kick"),
    ),
    request_body = KickMemberRequest,
    responses(
        (status = 204, description = "Member kicked"),
        (status = 400, description = "Invalid Guild ID or User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guild-members"
)]
pub async fn kick_member(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path((guild_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<KickMemberRequest>,
) -> Result<StatusCode, ApiError> {
    request.validate()?;
    state
        .guild
        .member_service
        .kick_member(guild_id, user_id, requester_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE `/api/v1/guilds/:guild_id/members/@me`
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}/members/@me",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 204, description = "Left guild"),
        (status = 400, description = "Invalid Guild ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Owner cannot leave"),
        (status = 404, description = "Guild not found")
    ),
    tag = "guild-members"
)]
pub async fn leave_guild(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(guild_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .guild
        .member_service
        .leave_guild(guild_id, user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT `/api/v1/guilds/:guild_id/members/:user_id/roles`
#[utoipa::path(
    put,
    path = "/api/v1/guilds/{guild_id}/members/{user_id}/roles",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned", body = GuildMemberResponse),
        (status = 400, description = "Invalid Guild ID or User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guild-members"
)]
pub async fn assign_role(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path((guild_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AssignRoleRequest>,
) -> Result<Json<GuildMemberResponse>, ApiError> {
    let member = state
        .guild
        .member_service
        .assign_role(guild_id, user_id, request.role_id, requester_id)
        .await?;
    Ok(Json(GuildMemberResponse::from(member)))
}

/// DELETE `/api/v1/guilds/:guild_id/members/:user_id/roles/:role_id`
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}/members/{user_id}/roles/{role_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role removed", body = GuildMemberResponse),
        (status = 400, description = "Invalid Guild ID, User ID or Role ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Guild, Member or Role not found")
    ),
    tag = "guild-members"
)]
pub async fn remove_role(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path((guild_id, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<GuildMemberResponse>, ApiError> {
    let member = state
        .guild
        .member_service
        .remove_role(guild_id, user_id, role_id, requester_id)
        .await?;
    Ok(Json(GuildMemberResponse::from(member)))
}
