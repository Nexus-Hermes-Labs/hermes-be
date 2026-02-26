#![allow(clippy::doc_markdown)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::presentation::dto::conversation::request::{
    AddMemberRequest, CreateGroupDmRequest, OpenDmRequest,
};
use crate::presentation::dto::conversation::response::{
    ConversationListResponse, ConversationMemberResponse, ConversationResponse,
};
use crate::state::AppState;
use common::middleware::authentication::RequestUser;

use super::error::ApiError;

/// GET /api/v1/conversations/@me — list authenticated user's conversations
#[utoipa::path(
    get,
    path = "/api/v1/conversations/@me",
    responses(
        (status = 200, description = "Conversations retrieved", body = ConversationListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "conversations"
)]
pub async fn get_my_conversations(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
) -> Result<Json<ConversationListResponse>, ApiError> {
    let conversations = state
        .messaging
        .conversation_service
        .get_user_conversations(user_id)
        .await?;

    let total = conversations.len();
    Ok(Json(ConversationListResponse {
        conversations: conversations
            .into_iter()
            .map(ConversationResponse::from)
            .collect(),
        total,
    }))
}

/// GET /api/v1/conversations/:conversation_id
#[utoipa::path(
    get,
    path = "/api/v1/conversations/{conversation_id}",
    params(("conversation_id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 200, description = "Conversation retrieved", body = ConversationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Conversation not found"),
    ),
    tag = "conversations"
)]
pub async fn get_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conv = state
        .messaging
        .conversation_service
        .get_conversation(conversation_id)
        .await?;
    Ok(Json(ConversationResponse::from(conv)))
}

/// POST /api/v1/conversations/dm — open or retrieve a DM
#[utoipa::path(
    post,
    path = "/api/v1/conversations/dm",
    request_body = OpenDmRequest,
    responses(
        (status = 200, description = "DM opened", body = ConversationResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "conversations"
)]
pub async fn open_dm(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Json(request): Json<OpenDmRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conv = state
        .messaging
        .conversation_service
        .open_dm(requester_id, request.target_user_id)
        .await?;
    Ok(Json(ConversationResponse::from(conv)))
}

/// POST /api/v1/conversations/group — create a group DM
#[utoipa::path(
    post,
    path = "/api/v1/conversations/group",
    request_body = CreateGroupDmRequest,
    responses(
        (status = 201, description = "Group DM created", body = ConversationResponse),
        (status = 400, description = "Invalid member count"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed"),
    ),
    tag = "conversations"
)]
pub async fn create_group_dm(
    State(state): State<AppState>,
    RequestUser {
        id: requester_id, ..
    }: RequestUser,
    Json(request): Json<CreateGroupDmRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), ApiError> {
    request.validate()?;

    // Include the requester in the member list
    let mut member_ids = request.member_ids;
    if !member_ids.contains(&requester_id) {
        member_ids.push(requester_id);
    }

    let conv = state
        .messaging
        .conversation_service
        .create_group_dm(member_ids)
        .await?;

    Ok((StatusCode::CREATED, Json(ConversationResponse::from(conv))))
}

/// GET /api/v1/conversations/:conversation_id/members
#[utoipa::path(
    get,
    path = "/api/v1/conversations/{conversation_id}/members",
    params(("conversation_id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 200, description = "Members retrieved", body = Vec<ConversationMemberResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Conversation not found"),
    ),
    tag = "conversations"
)]
pub async fn get_conversation_members(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ConversationMemberResponse>>, ApiError> {
    let members = state
        .messaging
        .conversation_service
        .get_conversation_members(conversation_id)
        .await?;

    Ok(Json(
        members
            .into_iter()
            .map(ConversationMemberResponse::from)
            .collect(),
    ))
}

/// POST /api/v1/conversations/:conversation_id/members
#[utoipa::path(
    post,
    path = "/api/v1/conversations/{conversation_id}/members",
    params(("conversation_id" = Uuid, Path, description = "Conversation ID")),
    request_body = AddMemberRequest,
    responses(
        (status = 204, description = "Member added"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Conversation not found"),
        (status = 409, description = "Already a member"),
    ),
    tag = "conversations"
)]
pub async fn add_member(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .messaging
        .conversation_service
        .add_member(conversation_id, request.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/conversations/:conversation_id/members/@me
#[utoipa::path(
    delete,
    path = "/api/v1/conversations/{conversation_id}/members/@me",
    params(("conversation_id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 204, description = "Left the conversation"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Conversation not found"),
    ),
    tag = "conversations"
)]
pub async fn leave_conversation(
    State(state): State<AppState>,
    RequestUser { id: user_id, .. }: RequestUser,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .messaging
        .conversation_service
        .remove_member(conversation_id, user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
