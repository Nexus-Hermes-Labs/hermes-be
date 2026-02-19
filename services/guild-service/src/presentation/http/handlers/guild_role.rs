use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::guild_role::{Permissions, RoleColor};
use crate::presentation::dto::guild_role::request::*;
use crate::presentation::dto::guild_role::response::*;
use crate::state::AppState;
use common::middleware::authentication::AuthenticatedUser;

use super::error::ApiError;

/// POST /api/v1/guilds/:guild_id/roles
#[utoipa::path(
    post,
    path = "/api/v1/guilds/{guild_id}/roles",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = GuildRoleResponse),
        (status = 403, description = "Forbidden"),
    ),
    tag = "guild-roles"
)]
pub async fn create_role(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(guild_id): Path<Uuid>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<GuildRoleResponse>), ApiError> {
    request.validate()?;
    let requester_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    let role = state
        .guild
        .role_service
        .create_role(
            guild_id,
            requester_id,
            request.name,
            Permissions::new(request.permissions),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(GuildRoleResponse::from(role))))
}

/// GET /api/v1/guilds/:guild_id/roles
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/roles",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 200, description = "Roles listed", body = Vec<GuildRoleResponse>),
        (status = 404, description = "Guild not found"),
    ),
    tag = "guild-roles"
)]
pub async fn list_roles(
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
) -> Result<Json<Vec<GuildRoleResponse>>, ApiError> {
    let roles = state.guild.role_service.list_roles(guild_id).await?;
    Ok(Json(roles.into_iter().map(GuildRoleResponse::from).collect()))
}

/// GET /api/v1/guilds/:guild_id/roles/:role_id
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/roles/{role_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role found", body = GuildRoleResponse),
        (status = 404, description = "Role not found"),
    ),
    tag = "guild-roles"
)]
pub async fn get_role(
    State(state): State<AppState>,
    Path((_guild_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GuildRoleResponse>, ApiError> {
    let role = state.guild.role_service.get_role(role_id).await?;
    Ok(Json(GuildRoleResponse::from(role)))
}

/// PATCH /api/v1/guilds/:guild_id/roles/:role_id
#[utoipa::path(
    patch,
    path = "/api/v1/guilds/{guild_id}/roles/{role_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = GuildRoleResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
    ),
    tag = "guild-roles"
)]
pub async fn update_role(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<GuildRoleResponse>, ApiError> {
    request.validate()?;
    let requester_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;

    let color = request.color.map(RoleColor);
    let permissions = request.permissions.map(Permissions::new);

    let role = state
        .guild
        .role_service
        .update_role(
            guild_id,
            role_id,
            requester_id,
            request.name,
            color,
            permissions,
            request.hoist,
            request.mentionable,
        )
        .await?;
    Ok(Json(GuildRoleResponse::from(role)))
}

/// DELETE /api/v1/guilds/:guild_id/roles/:role_id
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}/roles/{role_id}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
    ),
    tag = "guild-roles"
)]
pub async fn delete_role(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let requester_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    state.guild.role_service.delete_role(guild_id, role_id, requester_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
