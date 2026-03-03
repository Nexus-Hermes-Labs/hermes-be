#![allow(clippy::doc_markdown)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::presentation::dto::reaction::response::ReactionResponse;
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// PUT /api/v1/messages/{message_id}/reactions/{emoji}
#[utoipa::path(
    put,
    path = "/api/v1/messages/{message_id}/reactions/{emoji}",
    params(
        ("message_id" = Uuid, Path, description = "Message ID"),
        ("emoji" = String, Path, description = "Emoji string")
    ),
    responses(
        (status = 201, description = "Reaction added", body = ReactionResponse),
        (status = 400, description = "Invalid emoji"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Message not found"),
        (status = 409, description = "Already reacted"),
    ),
    tag = "reactions"
)]
pub async fn add_reaction(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path((message_id, emoji)): Path<(Uuid, String)>,
) -> Result<(StatusCode, Json<ReactionResponse>), ApiError> {
    if emoji.is_empty() || emoji.len() > 32 {
        return Err(ApiError::bad_request("Emoji must be 1–32 characters"));
    }

    let reaction = state
        .chat
        .reaction_service
        .add_reaction(message_id, user_id, emoji)
        .await?;

    Ok((StatusCode::CREATED, Json(ReactionResponse::from(reaction))))
}

/// DELETE /api/v1/messages/{message_id}/reactions/{emoji}
#[utoipa::path(
    delete,
    path = "/api/v1/messages/{message_id}/reactions/{emoji}",
    params(
        ("message_id" = Uuid, Path, description = "Message ID"),
        ("emoji" = String, Path, description = "Emoji string")
    ),
    responses(
        (status = 204, description = "Reaction removed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Reaction not found"),
    ),
    tag = "reactions"
)]
pub async fn remove_reaction(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path((message_id, emoji)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .chat
        .reaction_service
        .remove_reaction(message_id, user_id, &emoji)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/messages/{message_id}/reactions
#[utoipa::path(
    get,
    path = "/api/v1/messages/{message_id}/reactions",
    params(("message_id" = Uuid, Path, description = "Message ID")),
    responses(
        (status = 200, description = "Reactions retrieved", body = Vec<ReactionResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Message not found"),
    ),
    tag = "reactions"
)]
pub async fn get_reactions(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<ReactionResponse>>, ApiError> {
    let reactions = state
        .chat
        .reaction_service
        .get_reactions(message_id)
        .await?;

    Ok(Json(reactions.into_iter().map(ReactionResponse::from).collect()))
}
