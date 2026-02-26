use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::guild::GuildVisibility;
use crate::presentation::dto::guild::request::{
    CreateGuildRequest, SearchGuildsRequest, UpdateGuildRequest,
};
use crate::presentation::dto::guild::response::{
    CreateGuildResponse, GuildListResponse, GuildResponse,
};
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// POST /api/v1/guilds
#[utoipa::path(
    post,
    path = "/api/v1/guilds",
    request_body = CreateGuildRequest,
    responses(
        (status = 201, description = "Guild created with default channels", body = CreateGuildResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guilds"
)]
pub async fn create_guild(
    State(state): State<AppState>,
    RequestUser { id: owner_id, .. }: RequestUser,
    Json(request): Json<CreateGuildRequest>,
) -> Result<(StatusCode, Json<CreateGuildResponse>), ApiError> {
    request.validate()?;

    if request.name.contains('\0') {
        return Err(ApiError::bad_request(
            "Guild name contains invalid characters",
        ));
    }

    let guild = state
        .guild
        .guild_service
        .create_guild(owner_id, request.name, request.description)
        .await?;

    let guild_id = guild.id();

    // Ask channel-service to create default text + voice channels via gRPC
    let channels = state
        .shared
        .channel_grpc_client
        .create_default_channels(guild_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create default channels: {e}")))?;

    let first_channel_id = channels
        .text_channel
        .as_ref()
        .and_then(|c| c.id.parse::<Uuid>().ok())
        .ok_or_else(|| ApiError::internal("channel-service returned no text channel"))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateGuildResponse {
            guild: GuildResponse::from(guild),
            first_channel_id,
        }),
    ))
}

/// GET /`api/v1/guilds/:guild_id`
#[utoipa::path(
    get,
    path = "/api/v1/guilds/{guild_id}",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 200, description = "Guild found", body = GuildResponse),
        (status = 400, description = "Invalid Guild ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Guild not found"),
    ),
    tag = "guilds"
)]
pub async fn get_guild(
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
) -> Result<Json<GuildResponse>, ApiError> {
    let guild = state.guild.guild_service.get_guild(guild_id).await?;
    Ok(Json(GuildResponse::from(guild)))
}

/// PATCH /`api/v1/guilds/:guild_id`
#[utoipa::path(
    patch,
    path = "/api/v1/guilds/{guild_id}",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    request_body = UpdateGuildRequest,
    responses(
        (status = 200, description = "Guild updated", body = GuildResponse),
        (status = 400, description = "Invalid Guild ID or input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Guild not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guilds"
)]
pub async fn update_guild(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path(guild_id): Path<Uuid>,
    Json(request): Json<UpdateGuildRequest>,
) -> Result<Json<GuildResponse>, ApiError> {
    request.validate()?;

    if let Some(ref name) = request.name {
        if name.contains('\0') {
            return Err(ApiError::bad_request(
                "Guild name contains invalid characters",
            ));
        }
    }

    if let Some(ref desc) = request.description {
        if desc.contains('\0') {
            return Err(ApiError::bad_request(
                "Guild description contains invalid characters",
            ));
        }
    }

    let visibility = request
        .visibility
        .map(|v| v.parse::<GuildVisibility>().map_err(ApiError::bad_request))
        .transpose()?;

    let guild = state
        .guild
        .guild_service
        .update_guild(
            guild_id,
            requester_id,
            request.name,
            request.description,
            request.icon_url,
            request.banner_url,
            visibility,
        )
        .await?;

    Ok(Json(GuildResponse::from(guild)))
}

/// DELETE /`api/v1/guilds/:guild_id`
#[utoipa::path(
    delete,
    path = "/api/v1/guilds/{guild_id}",
    params(("guild_id" = Uuid, Path, description = "Guild ID")),
    responses(
        (status = 204, description = "Guild deleted"),
        (status = 400, description = "Invalid Guild ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Guild not found"),
    ),
    tag = "guilds"
)]
pub async fn delete_guild(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path(guild_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .guild
        .guild_service
        .delete_guild(guild_id, requester_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/guilds/@me — guilds the authenticated user is a member of
#[utoipa::path(
    get,
    path = "/api/v1/guilds/@me",
    responses(
        (status = 200, description = "User's guilds", body = GuildListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "guilds"
)]
pub async fn get_my_guilds(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
) -> Result<Json<GuildListResponse>, ApiError> {
    let guild_ids = state.guild.member_service.get_user_guilds(user_id).await?;

    let mut guilds = Vec::with_capacity(guild_ids.len());
    for id in guild_ids {
        if let Ok(g) = state.guild.guild_service.get_guild(id).await {
            guilds.push(GuildResponse::from(g));
        }
    }

    let total = i64::try_from(guilds.len()).unwrap_or(i64::MAX);
    Ok(Json(GuildListResponse {
        guilds,
        total,
        limit: total,
        offset: 0,
    }))
}

/// GET /api/v1/guilds/search?query=...&limit=20&offset=0
#[utoipa::path(
    get,
    path = "/api/v1/guilds/search",
    params(SearchGuildsRequest),
    responses(
        (status = 200, description = "Guilds found", body = GuildListResponse),
        (status = 400, description = "Missing query or invalid characters"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    tag = "guilds"
)]
pub async fn search_guilds(
    State(state): State<AppState>,
    Query(request): Query<SearchGuildsRequest>,
) -> Result<Json<GuildListResponse>, ApiError> {
    request.validate()?;

    if request.query.contains('\0') {
        return Err(ApiError::bad_request(
            "Search query contains invalid characters",
        ));
    }

    let guilds = state
        .guild
        .guild_service
        .search_guilds(request.query, request.limit, request.offset)
        .await?;

    let total = i64::try_from(guilds.len()).unwrap_or(i64::MAX);
    Ok(Json(GuildListResponse {
        guilds: guilds.into_iter().map(GuildResponse::from).collect(),
        total,
        limit: request.limit,
        offset: request.offset,
    }))
}
