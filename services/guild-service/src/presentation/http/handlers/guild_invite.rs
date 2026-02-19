use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::presentation::dto::guild_invite::request::*;
use crate::presentation::dto::guild_invite::response::*;
use crate::presentation::dto::guild_member::response::GuildMemberResponse;
use crate::state::AppState;
use common::middleware::authentication::AuthenticatedUser;

use super::error::ApiError;

/// POST /api/v1/guilds/:guild_id/invites
#[utoipa::path(
    post,
    path = "/api/v1/guilds/{guild_id}/invites",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite created", body = InviteResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Guild not found"),
    ),
    tag = "guild-invites"
)]
pub async fn create_invite(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(guild_id): Path<Uuid>,
    Json(request): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<InviteResponse>), ApiError> {
    let creator_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    let invite = state
        .guild
        .invite_service
        .create_invite(guild_id, creator_id, request.max_uses, request.max_age_seconds)
        .await?;
    Ok((StatusCode::CREATED, Json(InviteResponse::from(invite))))
}

/// GET /api/v1/invites/:code
#[utoipa::path(
    get,
    path = "/api/v1/invites/{code}",
    params(("code" = String, Path, description = "Invite code")),
    responses(
        (status = 200, description = "Invite found", body = InviteResponse),
        (status = 404, description = "Invite not found"),
    ),
    tag = "guild-invites"
)]
pub async fn get_invite(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<InviteResponse>, ApiError> {
    let invite = state.guild.invite_service.get_invite(code).await?;
    Ok(Json(InviteResponse::from(invite)))
}

/// POST /api/v1/invites/:code/use
#[utoipa::path(
    post,
    path = "/api/v1/invites/{code}/use",
    params(("code" = String, Path, description = "Invite code")),
    responses(
        (status = 200, description = "Joined guild", body = GuildMemberResponse),
        (status = 400, description = "Invite invalid or exhausted"),
        (status = 409, description = "Already a member"),
    ),
    tag = "guild-invites"
)]
pub async fn use_invite(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(code): Path<String>,
) -> Result<Json<GuildMemberResponse>, ApiError> {
    let user_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    let member = state.guild.invite_service.use_invite(code, user_id).await?;
    Ok(Json(GuildMemberResponse::from(member)))
}

/// DELETE /api/v1/guilds/:guild_id/invites/:code
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}/invites/{code}",
    params(
        ("guild_id" = Uuid, Path, description = "Guild ID"),
        ("code" = String, Path, description = "Invite code"),
    ),
    responses(
        (status = 204, description = "Invite revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invite not found"),
    ),
    tag = "guild-invites"
)]
pub async fn revoke_invite(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((_guild_id, code)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    let requester_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    state.guild.invite_service.revoke_invite(code, requester_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/guilds/:guild_id/invites
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/invites",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 200, description = "Invites listed", body = Vec<InviteResponse>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Guild not found"),
    ),
    tag = "guild-invites"
)]
pub async fn list_invites(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(guild_id): Path<Uuid>,
) -> Result<Json<Vec<InviteResponse>>, ApiError> {
    let requester_id = auth_user.0.user_id().map_err(|_| ApiError::forbidden("Invalid user ID"))?;
    let invites = state.guild.invite_service.list_invites(guild_id, requester_id).await?;
    Ok(Json(invites.into_iter().map(InviteResponse::from).collect()))
}
