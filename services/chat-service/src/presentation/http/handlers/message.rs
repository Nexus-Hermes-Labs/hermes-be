#![allow(clippy::doc_markdown)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::presentation::dto::message::request::{
    CreateMessageRequest, EditMessageRequest, GetMessagesQuery,
};
use crate::presentation::dto::message::response::{MessageListResponse, MessageResponse};
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// GET /api/v1/channels/{channel_id}/messages
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}/messages",
    params(
        ("channel_id" = Uuid, Path, description = "Channel ID"),
        GetMessagesQuery
    ),
    responses(
        (status = 200, description = "Messages retrieved", body = MessageListResponse),
        (status = 400, description = "Invalid channel ID"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "messages"
)]
pub async fn get_messages(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<MessageListResponse>, ApiError> {
    query.validate()?;

    let (messages, has_more) = state
        .chat
        .message_service
        .get_messages(channel_id, query.limit, query.before)
        .await?;

    Ok(Json(MessageListResponse {
        messages: messages.iter().map(MessageResponse::from).collect(),
        has_more,
    }))
}

/// POST /api/v1/channels/{channel_id}/messages
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/messages",
    params(("channel_id" = Uuid, Path, description = "Channel ID")),
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "Message sent", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed"),
    ),
    tag = "messages"
)]
pub async fn send_message(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(channel_id): Path<Uuid>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    request.validate()?;

    let message = state
        .chat
        .message_service
        .send_message(channel_id, user_id, request.content, request.reply_to_id)
        .await?;

    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

/// PATCH /api/v1/messages/{message_id}
#[utoipa::path(
    patch,
    path = "/api/v1/messages/{message_id}",
    params(("message_id" = Uuid, Path, description = "Message ID")),
    request_body = EditMessageRequest,
    responses(
        (status = 200, description = "Message updated", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Message not found"),
        (status = 422, description = "Validation failed"),
    ),
    tag = "messages"
)]
pub async fn edit_message(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    request.validate()?;

    let message = state
        .chat
        .message_service
        .edit_message(message_id, user_id, request.content)
        .await?;

    Ok(Json(MessageResponse::from(message)))
}

/// DELETE /api/v1/messages/{message_id}
#[utoipa::path(
    delete,
    path = "/api/v1/messages/{message_id}",
    params(("message_id" = Uuid, Path, description = "Message ID")),
    responses(
        (status = 204, description = "Message deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Message not found"),
    ),
    tag = "messages"
)]
pub async fn delete_message(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .chat
        .message_service
        .delete_message(message_id, user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
