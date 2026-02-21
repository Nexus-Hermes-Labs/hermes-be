use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;
use validator::Validate;
use crate::presentation::dto::user_relationship::{
    Pagination, RelationshipRequest, RelationshipResponse,
};
use crate::presentation::http::handlers::error::ApiError;
use crate::state::AppState;

// ============================================
// Handler
// ============================================

pub struct UserRelationshipHandler;

// POST /api/v1/users/{user_id}/relationships/request
#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/relationships/request",
    params(
        ("user_id" = Uuid, Path, description = "Sender User ID")
    ),
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "Friend request sent", body = RelationshipResponse),
        (status = 400, description = "Invalid User ID or input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Blocked or privacy settings prevent request"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn send_friend_request(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let relationship = state
        .user
        .relationship_service
        .send_friend_request(user_id, payload.target_user_id, payload.message)
        .await?;

    Ok(Json(RelationshipResponse {
        user_id: relationship.user_id(),
        target_user_id: relationship.target_user_id(),
        r#type: relationship.relationship_type().to_string(),
        message: relationship.message().map(|s| s.to_string()),
    }))
}

// PUT /api/v1/users/{user_id}/relationships/request/accept
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/relationships/request/accept",
    params(
        ("user_id" = Uuid, Path, description = "Recipient User ID")
    ),
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "Friend request accepted", body = RelationshipResponse),
        (status = 400, description = "Invalid User ID or input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship or User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn accept_friend_request(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let relationship = state
        .user
        .relationship_service
        .accept_friend_request(user_id, payload.target_user_id)
        .await?;

    Ok(Json(RelationshipResponse {
        user_id: relationship.user_id(),
        target_user_id: relationship.target_user_id(),
        r#type: relationship.relationship_type().to_string(),
        message: relationship.message().map(|s| s.to_string()),
    }))
}

// PUT /api/v1/users/{user_id}/relationships/request/decline
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/relationships/request/decline",
    params(
        ("user_id" = Uuid, Path, description = "Recipient User ID")
    ),
    request_body = RelationshipRequest,
    responses(
        (status = 204, description = "Friend request declined"),
        (status = 400, description = "Invalid User ID or input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship or User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn decline_friend_request(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<RelationshipRequest>,
) -> Result<StatusCode, ApiError> {
    payload.validate()?;
    state
        .user
        .relationship_service
        .decline_friend_request(user_id, payload.target_user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// DELETE /api/v1/users/{user_id}/relationships/friend/{target_user_id}
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/relationships/friend/{target_user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 204, description = "Friend removed"),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship or User not found")
    ),
    tag = "user-relationship"
)]
pub async fn remove_friend(
    State(state): State<AppState>,
    Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .user
        .relationship_service
        .remove_friend(user_id, target_user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/v1/users/{user_id}/relationships/block
#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/relationships/block",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "User blocked", body = RelationshipResponse),
        (status = 400, description = "Invalid User ID or input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Cannot block oneself"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn block_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let relationship = state
        .user
        .relationship_service
        .block_user(user_id, payload.target_user_id)
        .await?;

    Ok(Json(RelationshipResponse {
        user_id: relationship.user_id(),
        target_user_id: relationship.target_user_id(),
        r#type: relationship.relationship_type().to_string(),
        message: relationship.message().map(|s| s.to_string()),
    }))
}

// DELETE /api/v1/users/{user_id}/relationships/block/{target_user_id}
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/relationships/block/{target_user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 204, description = "User unblocked"),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship or User not found")
    ),
    tag = "user-relationship"
)]
pub async fn unblock_user(
    State(state): State<AppState>,
    Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .user
        .relationship_service
        .unblock_user(user_id, target_user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// GET /api/v1/users/{user_id}/relationships/{target_user_id}
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/relationships/{target_user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 200, description = "Relationship found", body = RelationshipResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship or User not found")
    ),
    tag = "user-relationship"
)]
pub async fn get_relationship(
    State(state): State<AppState>,
    Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    let relationship = state
        .user
        .relationship_service
        .get_relationship(user_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Relationship not found"))?;

    Ok(Json(RelationshipResponse {
        user_id: relationship.user_id(),
        target_user_id: relationship.target_user_id(),
        r#type: relationship.relationship_type().to_string(),
        message: relationship.message().map(|s| s.to_string()),
    }))
}

// GET /api/v1/users/{user_id}/relationships/friends
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/relationships/friends",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "Friends list", body = [RelationshipResponse]),
        (status = 400, description = "Invalid User ID or query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn get_friends(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let relationships = state
        .user
        .relationship_service
        .get_friends(user_id, pagination.limit, pagination.offset)
        .await?;

    let response = relationships
        .iter()
        .map(|r| RelationshipResponse {
            user_id: r.user_id(),
            target_user_id: r.target_user_id(),
            r#type: r.relationship_type().to_string(),
            message: r.message().map(|s| s.to_string()),
        })
        .collect();

    Ok(Json(response))
}

// GET /api/v1/users/{user_id}/relationships/incoming
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/relationships/incoming",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "Incoming friend requests", body = [RelationshipResponse]),
        (status = 400, description = "Invalid User ID or query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn get_incoming_requests(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let relationships = state
        .user
        .relationship_service
        .get_incoming_requests(user_id, pagination.limit, pagination.offset)
        .await?;

    let response = relationships
        .iter()
        .map(|r| RelationshipResponse {
            user_id: r.user_id(),
            target_user_id: r.target_user_id(),
            r#type: r.relationship_type().to_string(),
            message: r.message().map(|s| s.to_string()),
        })
        .collect();

    Ok(Json(response))
}

// GET /api/v1/users/{user_id}/relationships/outgoing
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/relationships/outgoing",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "Outgoing friend requests", body = [RelationshipResponse]),
        (status = 400, description = "Invalid User ID or query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn get_outgoing_requests(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let relationships = state
        .user
        .relationship_service
        .get_outgoing_requests(user_id, pagination.limit, pagination.offset)
        .await?;

    let response = relationships
        .iter()
        .map(|r| RelationshipResponse {
            user_id: r.user_id(),
            target_user_id: r.target_user_id(),
            r#type: r.relationship_type().to_string(),
            message: r.message().map(|s| s.to_string()),
        })
        .collect();

    Ok(Json(response))
}

// GET /api/v1/users/{user_id}/relationships/blocked
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/relationships/blocked",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "Blocked users list", body = [RelationshipResponse]),
        (status = 400, description = "Invalid User ID or query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-relationship"
)]
pub async fn get_blocked_users(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let relationships = state
        .user
        .relationship_service
        .get_blocked_users(user_id, pagination.limit, pagination.offset)
        .await?;

    let response = relationships
        .iter()
        .map(|r| RelationshipResponse {
            user_id: r.user_id(),
            target_user_id: r.target_user_id(),
            r#type: r.relationship_type().to_string(),
            message: r.message().map(|s| s.to_string()),
        })
        .collect();

    Ok(Json(response))
}
