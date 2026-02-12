use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use common::pagination::PaginationParams;
use common::AppError;
use common::dto::common::SearchQuery;
use common::middleware::authentication::AuthenticatedUser;
use crate::presentation::http::dto::user::{SetCustomStatusRequest, UpdatePrivacySettingsRequest, UpdateProfileRequest};
use crate::presentation::http::state::AppState;

// =====================================================
// PROFILE QUERIES
// =====================================================

/// Get current user_profile's own profile (includes private privacy settings)
///
/// GET /users/me
#[axum::debug_handler]
pub async fn get_my_profile_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let response = state.user_service.get_my_profile(claims.sub).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Get another user_profile's public profile (privacy-aware)
///
/// GET /users/:id
#[axum::debug_handler]
pub async fn get_user_profile_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let viewer_id = claims.sub;

    let response = state
        .user_service
        .get_user_profile(user_id, viewer_id)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Get user_profile by username
///
/// GET /users/username/:username
#[axum::debug_handler]
pub async fn get_user_by_username_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Response, AppError> {
    let viewer_id = claims.sub;

    let response = state
        .user_service
        .get_user_by_username(&username, viewer_id)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

// =====================================================
// PROFILE MANAGEMENT
// =====================================================

/// Update user_profile profile (display_name, avatar, banner, bio)
///
/// PATCH /users/me/profile
#[axum::debug_handler]
pub async fn update_profile_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = state
        .user_service
        .update_profile(claims.sub, request)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

// =====================================================
// PRIVACY SETTINGS
// =====================================================

/// Update privacy settings
///
/// PATCH /users/me/privacy
#[axum::debug_handler]
pub async fn update_privacy_settings_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<UpdatePrivacySettingsRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = state
        .user_service
        .update_privacy_settings(claims.sub, request)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

// =====================================================
// CUSTOM STATUS
// =====================================================

/// Set or update custom status
///
/// PUT /users/me/status
#[axum::debug_handler]
pub async fn set_custom_status_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<SetCustomStatusRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = state
        .user_service
        .set_custom_status(claims.sub, request)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Clear custom status
///
/// DELETE /users/me/status
#[axum::debug_handler]
pub async fn clear_custom_status_handler(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    state.user_service.clear_custom_status(claims.sub).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}


/// Search users by username, display_name, or bio
///
/// GET /users/search?q=alice&page=1&page_size=20
#[axum::debug_handler]
pub async fn search_users_handler(
    AuthenticatedUser(_claims): AuthenticatedUser, // Require auth but don't use claims
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Response, AppError> {
    // Validate query
    query
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let params = PaginationParams::new(query.page, query.page_size);

    let response = state.user_service.search_users(&query.q, params).await?;

    Ok((StatusCode::OK, Json(response)).0.into_response())
}