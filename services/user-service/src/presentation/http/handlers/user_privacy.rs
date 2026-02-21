use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::domain::user_privacy::{
    ContentFilterLevel, DmPrivacy, FriendRequestPrivacy, PrivacyPreset,
};
use crate::presentation::dto::user_privacy::*;
use crate::state::AppState;

use super::error::ApiError;

/// HTTP handlers for user privacy operations
pub struct UserPrivacyHandler;

// ============================================
// PRIVACY SETTINGS
// ============================================

/// GET /users/:user_id/privacy
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/privacy",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Privacy settings found", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found")
    ),
    tag = "user-privacy"
)]
pub async fn get_privacy_settings(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let service = &state.user.user_privacy_service;
    let settings = service.get_privacy_settings(user_id).await?;
    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// PUT /users/:user_id/privacy/dm
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/privacy/dm",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateDmPrivacyRequest,
    responses(
        (status = 200, description = "DM privacy updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-privacy"
)]
pub async fn update_dm_privacy(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateDmPrivacyRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let privacy = match request.allow_dms_from {
        DmPrivacyInput::Everyone => DmPrivacy::Everyone,
        DmPrivacyInput::Friends => DmPrivacy::Friends,
        DmPrivacyInput::ServerMembers => DmPrivacy::ServerMembers,
        DmPrivacyInput::None => DmPrivacy::None,
    };

    let service = &state.user.user_privacy_service;
    let settings = service.update_dm_privacy(user_id, privacy).await?;

    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// PUT /users/:user_id/privacy/friend-requests
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/privacy/friend-requests",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateFriendRequestPrivacyRequest,
    responses(
        (status = 200, description = "Friend request privacy updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-privacy"
)]
pub async fn update_friend_request_privacy(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateFriendRequestPrivacyRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let privacy = match request.allow_friend_requests_from {
        FriendRequestPrivacyInput::Everyone => FriendRequestPrivacy::Everyone,
        FriendRequestPrivacyInput::FriendsOfFriends => FriendRequestPrivacy::FriendsOfFriends,
        FriendRequestPrivacyInput::None => FriendRequestPrivacy::None,
    };

    let service = &state.user.user_privacy_service;
    let settings = service
        .update_friend_request_privacy(user_id, privacy)
        .await?;

    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// PATCH /users/:user_id/privacy/visibility
#[utoipa::path(
    patch,
    path = "/api/v1/users/{user_id}/privacy/visibility",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateVisibilityRequest,
    responses(
        (status = 200, description = "Visibility updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-privacy"
)]
pub async fn update_visibility(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateVisibilityRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let service = &state.user.user_privacy_service;
    let settings = service
        .update_visibility(
            user_id,
            request.show_online_status,
            request.show_current_activity,
            request.show_profile_to_non_friends,
        )
        .await?;

    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// PATCH /users/:user_id/privacy/content
#[utoipa::path(
    patch,
    path = "/api/v1/users/{user_id}/privacy/content",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateContentSettingsRequest,
    responses(
        (status = 200, description = "Content settings updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-privacy"
)]
pub async fn update_content_settings(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateContentSettingsRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let filter_level = request
        .content_filter_level
        .map(|level| {
            ContentFilterLevel::from_i16(level).map_err(|_| {
                ApiError::validation(format!("Invalid content filter level: {}", level))
            })
        })
        .transpose()?;

    let service = &state.user.user_privacy_service;
    let settings = service
        .update_content_settings(user_id, request.allow_nsfw_content, filter_level)
        .await?;

    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// POST /users/:user_id/privacy/preset
#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/privacy/preset",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = ApplyPresetRequest,
    responses(
        (status = 200, description = "Privacy preset applied", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Privacy settings not found"),
        (status = 422, description = "Validation failed")
    ),
    tag = "user-privacy"
)]
pub async fn apply_preset(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ApplyPresetRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let preset = match request.preset {
        PrivacyPresetInput::Public => PrivacyPreset::Public,
        PrivacyPresetInput::FriendsOnly => PrivacyPreset::FriendsOnly,
        PrivacyPresetInput::Private => PrivacyPreset::Private,
    };

    let service = &state.user.user_privacy_service;
    let settings = service.apply_preset(user_id, preset).await?;

    Ok(Json(PrivacySettingsResponse::from(settings)))
}
