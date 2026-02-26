#![allow(clippy::doc_markdown)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::presentation::dto::message::request::{
    EditMessageRequest, GetMessagesQuery, SendMessageRequest,
};
use crate::presentation::dto::message::response::{MessageListResponse, MessageResponse};
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// GET /api/v1/channels/:channel_id/messages
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}/messages",
    params(
        ("channel_id" = Uuid, Path, description = "Channel ID"),
        GetMessagesQuery
    ),
    responses(
        (status = 200, description = "Messages retrieved", body = MessageListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "messages"
)]
pub async fn get_channel_messages(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<MessageListResponse>, ApiError> {
    query.validate()?;

    let limit = query.limit;
    let messages = state
        .messaging
        .message_service
        .get_channel_messages(channel_id, limit, query.before_id)
        .await?;

    let has_more = i64::try_from(messages.len()).unwrap_or(i64::MAX) == limit;
    Ok(Json(MessageListResponse {
        messages: messages.into_iter().map(MessageResponse::from).collect(),
        has_more,
    }))
}

/// POST /api/v1/channels/:channel_id/messages
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/messages",
    params(("channel_id" = Uuid, Path, description = "Channel ID")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed"),
    ),
    tag = "messages"
)]
pub async fn send_channel_message(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(channel_id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    request.validate()?;

    let message = state
        .messaging
        .message_service
        .send_channel_message(channel_id, user_id, request.content, request.reply_to_id)
        .await?;

    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

/// GET /api/v1/conversations/:conversation_id/messages
#[utoipa::path(
    get,
    path = "/api/v1/conversations/{conversation_id}/messages",
    params(
        ("conversation_id" = Uuid, Path, description = "Conversation ID"),
        GetMessagesQuery
    ),
    responses(
        (status = 200, description = "Messages retrieved", body = MessageListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "messages"
)]
pub async fn get_conversation_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<MessageListResponse>, ApiError> {
    query.validate()?;

    let limit = query.limit;
    let messages = state
        .messaging
        .message_service
        .get_conversation_messages(conversation_id, limit, query.before_id)
        .await?;

    let has_more = i64::try_from(messages.len()).unwrap_or(i64::MAX) == limit;
    Ok(Json(MessageListResponse {
        messages: messages.into_iter().map(MessageResponse::from).collect(),
        has_more,
    }))
}

/// POST /api/v1/conversations/:conversation_id/messages
#[utoipa::path(
    post,
    path = "/api/v1/conversations/{conversation_id}/messages",
    params(("conversation_id" = Uuid, Path, description = "Conversation ID")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed"),
    ),
    tag = "messages"
)]
pub async fn send_conversation_message(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    request.validate()?;

    let message = state
        .messaging
        .message_service
        .send_conversation_message(
            conversation_id,
            user_id,
            request.content,
            request.reply_to_id,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

/// PATCH /api/v1/messages/:message_id
#[utoipa::path(
    patch,
    path = "/api/v1/messages/{message_id}",
    params(("message_id" = Uuid, Path, description = "Message ID")),
    request_body = EditMessageRequest,
    responses(
        (status = 200, description = "Message edited", body = MessageResponse),
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
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    request.validate()?;

    let message = state
        .messaging
        .message_service
        .edit_message(message_id, requester_id, request.content)
        .await?;

    Ok(Json(MessageResponse::from(message)))
}

/// DELETE /api/v1/messages/:message_id
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
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .messaging
        .message_service
        .delete_message(message_id, requester_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
