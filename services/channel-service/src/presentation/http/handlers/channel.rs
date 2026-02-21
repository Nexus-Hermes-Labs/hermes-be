use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::presentation::dto::channel::request::{CreateChannelRequest, UpdateChannelRequest};
use crate::presentation::dto::channel::response::{ChannelListResponse, ChannelResponse};
use crate::state::AppState;
use common::middleware::authentication::AuthenticatedUser;

use super::error::ApiError;

/// POST `/api/v1/guilds/{guild_id}/channels`
#[utoipa::path(
    post,
    path = "/api/v1/guilds/{guild_id}/channels",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    request_body = CreateChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = ChannelResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only guild owner can create channels"),
        (status = 404, description = "Guild not found"),
        (status = 422, description = "Malformed JSON or wrong field type"),
        (status = 503, description = "Guild service unavailable"),
    ),
    tag = "channels"
)]
pub async fn create_channel(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(guild_id): Path<Uuid>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<ChannelResponse>), ApiError> {
    request.validate()?;

    let requester_id = auth_user
        .0
        .user_id()
        .map_err(|_| ApiError::forbidden("Invalid user ID"))?;

    let channel = state
        .channel
        .channel_service
        .create_channel(
            guild_id,
            requester_id,
            request.name,
            request.channel_type,
            request.parent_id,
            request.description,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(ChannelResponse::from(channel))))
}

/// GET `/api/v1/guilds/{guild_id}/channels`
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}/channels",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 200, description = "Channels listed", body = ChannelListResponse),
        (status = 404, description = "Guild not found"),
    ),
    tag = "channels"
)]
pub async fn list_guild_channels(
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
) -> Result<Json<ChannelListResponse>, ApiError> {
    let channels = state
        .channel
        .channel_service
        .list_guild_channels(guild_id)
        .await?;

    let total = i64::try_from(channels.len()).unwrap_or(i64::MAX);
    Ok(Json(ChannelListResponse {
        channels: channels.into_iter().map(ChannelResponse::from).collect(),
        total,
    }))
}

/// GET `/api/v1/channels/{channel_id}`
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}",
    params(("channel_id" = Uuid, Path, description = "Channel ID")),
    responses(
        (status = 200, description = "Channel found", body = ChannelResponse),
        (status = 404, description = "Channel not found"),
    ),
    tag = "channels"
)]
pub async fn get_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let channel = state
        .channel
        .channel_service
        .get_channel(channel_id)
        .await?;
    Ok(Json(ChannelResponse::from(channel)))
}

/// PATCH `/api/v1/channels/{channel_id}`
#[utoipa::path(
    patch,
    path = "/api/v1/channels/{channel_id}",
    params(("channel_id" = Uuid, Path, description = "Channel ID")),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Channel updated", body = ChannelResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only guild owner can update channels"),
        (status = 404, description = "Channel not found"),
        (status = 422, description = "Malformed JSON or wrong field type"),
        (status = 503, description = "Guild service unavailable"),
    ),
    tag = "channels"
)]
pub async fn update_channel(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(channel_id): Path<Uuid>,
    Json(request): Json<UpdateChannelRequest>,
) -> Result<Json<ChannelResponse>, ApiError> {
    request.validate()?;

    let requester_id = auth_user
        .0
        .user_id()
        .map_err(|_| ApiError::forbidden("Invalid user ID"))?;

    let channel = state
        .channel
        .channel_service
        .update_channel(
            channel_id,
            requester_id,
            request.name,
            request.parent_id,
            request.description,
            request.position,
        )
        .await?;

    Ok(Json(ChannelResponse::from(channel)))
}

/// DELETE `/api/v1/channels/{channel_id}`
#[utoipa::path(
    delete,
    path = "/api/v1/channels/{channel_id}",
    params(("channel_id" = Uuid, Path, description = "Channel ID")),
    responses(
        (status = 204, description = "Channel deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only guild owner can delete channels"),
        (status = 404, description = "Channel not found"),
        (status = 503, description = "Guild service unavailable"),
    ),
    tag = "channels"
)]
pub async fn delete_channel(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(channel_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let requester_id = auth_user
        .0
        .user_id()
        .map_err(|_| ApiError::forbidden("Invalid user ID"))?;

    state
        .channel
        .channel_service
        .delete_channel(channel_id, requester_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
