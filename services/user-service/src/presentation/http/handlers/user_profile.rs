use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::user_profile::UserStatus;
use crate::presentation::dto::user_profile::{request::*, response::*};
use crate::state::AppState;
use common::middleware::authentication::AuthenticatedUser;

use super::error::ApiError;

/// HTTP handlers for user profile operations
pub struct UserProfileHandler;

// ============================================
// PROFILE MANAGEMENT
// ============================================

/// GET /users/:user_id
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Profile found", body = ProfileResponse),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    auth_user: Option<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let requester_id = auth_user.and_then(|u| u.0.user_id().ok());
    let service = &state.user.user_profile_service;
    let profile = service.get_profile(user_id, requester_id).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// GET /users/username/:username
#[utoipa::path(
    get,
    path = "/api/v1/users/username/{username}",
    params(
        ("username" = String, Path, description = "Username")
    ),
    responses(
        (status = 200, description = "Profile found", body = ProfileResponse),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn get_profile_by_username(
    State(state): State<AppState>,
    auth_user: Option<AuthenticatedUser>,
    Path(username): Path<String>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let requester_id = auth_user.and_then(|u| u.0.user_id().ok());
    let service = &state.user.user_profile_service;
    let profile = service.get_profile_by_username(username, requester_id).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// POST /users
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateProfileRequest,
    responses(
        (status = 201, description = "Profile created", body = ProfileResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Username already taken")
    ),
    tag = "user-profile"
)]
pub async fn create_profile(
    State(state): State<AppState>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<ProfileResponse>), ApiError> {
    request.validate()?;

    let service = &state.user.user_profile_service;
    let profile = service
        .create_profile(request.username, request.display_name)
        .await?;

    Ok((StatusCode::CREATED, Json(ProfileResponse::from(profile))))
}

/// PATCH /users/:user_id
#[utoipa::path(
    patch,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ProfileResponse),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;

    let service = &state.user.user_profile_service;
    let profile = service
        .update_profile(
            user_id,
            request.display_name,
            request.bio,
            request.avatar_url,
            request.banner_url,
        )
        .await?;

    Ok(Json(ProfileResponse::from(profile)))
}

/// PUT /users/:user_id/username
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/username",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = ChangeUsernameRequest,
    responses(
        (status = 200, description = "Username changed", body = ProfileResponse),
        (status = 404, description = "Profile not found"),
        (status = 409, description = "Username already taken")
    ),
    tag = "user-profile"
)]
pub async fn change_username(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ChangeUsernameRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;

    let service = &state.user.user_profile_service;
    let profile = service.change_username(user_id, request.new_username).await?;

    Ok(Json(ProfileResponse::from(profile)))
}

/// DELETE /users/:user_id
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "Profile deleted"),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn delete_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let service = &state.user.user_profile_service;
    service.delete_profile(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// PRESENCE
// ============================================

/// PUT /users/:user_id/status
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/status",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = ProfileResponse),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn update_status(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let status = request
        .status
        .parse::<UserStatus>()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let service = &state.user.user_profile_service;
    let profile = service.update_status(user_id, status).await?;

    Ok(Json(ProfileResponse::from(profile)))
}

/// PUT /users/:user_id/custom-status
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/custom-status",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = SetCustomStatusRequest,
    responses(
        (status = 200, description = "Custom status updated", body = ProfileResponse),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn set_custom_status(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<SetCustomStatusRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;

    let service = &state.user.user_profile_service;
    let profile = service
        .set_custom_status(user_id, request.text, request.emoji, request.expires_at)
        .await?;

    Ok(Json(ProfileResponse::from(profile)))
}

/// DELETE /users/:user_id/custom-status
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/custom-status",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "Custom status cleared"),
        (status = 404, description = "Profile not found")
    ),
    tag = "user-profile"
)]
pub async fn clear_custom_status(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let service = &state.user.user_profile_service;
    service.clear_custom_status(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// SEARCH & DISCOVERY
// ============================================

/// GET /users/search?query=...&limit=10&offset=0
#[utoipa::path(
    get,
    path = "/api/v1/users/search",
    params(
        SearchUsersRequest
    ),
    responses(
        (status = 200, description = "Users found", body = ProfileListResponse)
    ),
    tag = "user-profile"
)]
pub async fn search_users(
    State(state): State<AppState>,
    auth_user: Option<AuthenticatedUser>,
    Query(request): Query<SearchUsersRequest>,
) -> Result<Json<ProfileListResponse>, ApiError> {
    let requester_id = auth_user.and_then(|u| u.0.user_id().ok());
    let service = &state.user.user_profile_service;
    let profiles = service
        .search_users(request.query, request.limit, request.offset, requester_id)
        .await?;

    let total = profiles.len() as i64;
    let profile_responses = profiles.into_iter().map(ProfileResponse::from).collect();

    Ok(Json(ProfileListResponse {
        profiles: profile_responses,
        total,
        limit: request.limit,
        offset: request.offset,
    }))
}

/// GET /users/online?limit=10&offset=0
#[utoipa::path(
    get,
    path = "/api/v1/users/online",
    params(
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset")
    ),
    responses(
        (status = 200, description = "Online users found", body = OnlineUsersResponse)
    ),
    tag = "user-profile"
)]
pub async fn get_online_users(
    State(state): State<AppState>,
    auth_user: Option<AuthenticatedUser>,
    Query(request): Query<SearchUsersRequest>,
) -> Result<Json<OnlineUsersResponse>, ApiError> {
    let requester_id = auth_user.and_then(|u| u.0.user_id().ok());
    let service = &state.user.user_profile_service;
    let profiles = service
        .get_online_users(request.limit, request.offset, requester_id)
        .await?;

    let total = profiles.len() as i64;
    let users = profiles.into_iter().map(ProfileResponse::from).collect();

    Ok(Json(OnlineUsersResponse { users, total }))
}

/// GET /users/check-username/:username
#[utoipa::path(
    get,
    path = "/api/v1/users/check-username/{username}",
    params(
        ("username" = String, Path, description = "Username")
    ),
    responses(
        (status = 200, description = "Username availability checked", body = UsernameAvailabilityResponse)
    ),
    tag = "user-profile"
)]
pub async fn check_username_availability(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<UsernameAvailabilityResponse>, ApiError> {
    let service = &state.user.user_profile_service;
    let available = service.is_username_available(username.clone()).await?;

    Ok(Json(UsernameAvailabilityResponse {
        username,
        available,
    }))
}
