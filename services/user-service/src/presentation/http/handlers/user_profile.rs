use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::user_profile::UserStatus;
use crate::presentation::dto::*;
use crate::state::AppState;

use super::error::ApiError;

/// HTTP handlers for user profile operations
pub struct UserProfileHandler;

impl UserProfileHandler {
    // ============================================
    // PROFILE MANAGEMENT
    // ============================================

    /// GET /users/:user_id
    pub async fn get_profile(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
    ) -> Result<Json<ProfileResponse>, ApiError> {
        let service = &state.user.user_profile_service;
        let profile = service.get_profile(user_id).await?;
        Ok(Json(ProfileResponse::from(profile)))
    }

    /// GET /users/username/:username
    pub async fn get_profile_by_username(
        State(state): State<AppState>,
        Path(username): Path<String>,
    ) -> Result<Json<ProfileResponse>, ApiError> {
        let service = &state.user.user_profile_service;
        let profile = service.get_profile_by_username(username).await?;
        Ok(Json(ProfileResponse::from(profile)))
    }

    /// POST /users
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
    pub async fn search_users(
        State(state): State<AppState>,
        Query(request): Query<SearchUsersRequest>,
    ) -> Result<Json<ProfileListResponse>, ApiError> {
        let service = &state.user.user_profile_service;
        let profiles = service
            .search_users(request.query, request.limit, request.offset)
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
    pub async fn get_online_users(
        State(state): State<AppState>,
        Query(request): Query<SearchUsersRequest>,
    ) -> Result<Json<OnlineUsersResponse>, ApiError> {
        let service = &state.user.user_profile_service;
        let profiles = service
            .get_online_users(request.limit, request.offset)
            .await?;

        let total = profiles.len() as i64;
        let users = profiles.into_iter().map(ProfileResponse::from).collect();

        Ok(Json(OnlineUsersResponse { users, total }))
    }

    /// GET /users/check-username/:username
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
}
