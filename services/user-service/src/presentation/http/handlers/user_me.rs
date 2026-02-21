use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use common::middleware::authentication::AuthenticatedUser;

use crate::domain::user_privacy::{
    ContentFilterLevel, DmPrivacy, FriendRequestPrivacy, PrivacyPreset,
};
use crate::domain::user_profile::UserStatus;
use crate::presentation::dto::user_privacy::*;
use crate::presentation::dto::user_profile::{request::*, response::*};
use crate::presentation::dto::user_relationship::{
    Pagination, RelationshipRequest, RelationshipResponse,
};
use crate::state::AppState;

use super::error::ApiError;

/// HTTP handlers for `@me` routes — authenticated user operating on their own data
pub struct UserMeHandler;

// ============================================
// PROFILE
// ============================================

/// GET /@me
#[utoipa::path(
    get,
    path = "/api/v1/users/@me",
    responses(
        (status = 200, description = "Authenticated user profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_profile_service;
    let profile = service.get_profile(user_id, Some(user_id)).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// PATCH /@me
#[utoipa::path(
    patch,
    path = "/api/v1/users/@me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// DELETE /@me
#[utoipa::path(
    delete,
    path = "/api/v1/users/@me",
    responses(
        (status = 204, description = "Profile deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn delete_profile(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_profile_service;
    service.delete_profile(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /@me/username
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/username",
    request_body = ChangeUsernameRequest,
    responses(
        (status = 200, description = "Username changed", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
        (status = 409, description = "Username already taken"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn change_username(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<ChangeUsernameRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_profile_service;
    let profile = service
        .change_username(user_id, request.new_username)
        .await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// PUT /@me/status
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/status",
    request_body = UpdateStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = ProfileResponse),
        (status = 400, description = "Invalid JSON body"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_status(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let status = match request.status {
        UserStatusInput::Online => UserStatus::Online,
        UserStatusInput::Offline => UserStatus::Offline,
        UserStatusInput::Idle => UserStatus::Idle,
        UserStatusInput::Dnd => UserStatus::DoNotDisturb,
    };
    let service = &state.user.user_profile_service;
    let profile = service.update_status(user_id, status).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// PUT /@me/custom-status
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/custom-status",
    request_body = SetCustomStatusRequest,
    responses(
        (status = 200, description = "Custom status updated", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn set_custom_status(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<SetCustomStatusRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    request.validate()?;
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_profile_service;
    let profile = service
        .set_custom_status(user_id, request.text, request.emoji, request.expires_at)
        .await?;
    Ok(Json(ProfileResponse::from(profile)))
}

/// DELETE /@me/custom-status
#[utoipa::path(
    delete,
    path = "/api/v1/users/@me/custom-status",
    responses(
        (status = 204, description = "Custom status cleared"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn clear_custom_status(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_profile_service;
    service.clear_custom_status(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// PRIVACY
// ============================================

/// GET /@me/privacy
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/privacy",
    responses(
        (status = 200, description = "Authenticated user privacy settings", body = PrivacySettingsResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_privacy_settings(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let service = &state.user.user_privacy_service;
    let settings = service.get_privacy_settings(user_id).await?;
    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// PUT /@me/privacy/dm
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/privacy/dm",
    request_body = UpdateDmPrivacyRequest,
    responses(
        (status = 200, description = "DM privacy updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid JSON body"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_dm_privacy(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateDmPrivacyRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
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

/// PUT /@me/privacy/friend-requests
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/privacy/friend-requests",
    request_body = UpdateFriendRequestPrivacyRequest,
    responses(
        (status = 200, description = "Friend request privacy updated", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid JSON body"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_friend_request_privacy(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateFriendRequestPrivacyRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
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

/// PATCH /@me/privacy/visibility
#[utoipa::path(
    patch,
    path = "/api/v1/users/@me/privacy/visibility",
    request_body = UpdateVisibilityRequest,
    responses(
        (status = 200, description = "Visibility updated", body = PrivacySettingsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_visibility(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateVisibilityRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
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

/// PATCH /@me/privacy/content
#[utoipa::path(
    patch,
    path = "/api/v1/users/@me/privacy/content",
    request_body = UpdateContentSettingsRequest,
    responses(
        (status = 200, description = "Content settings updated", body = PrivacySettingsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn update_content_settings(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<UpdateContentSettingsRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let filter_level = request
        .content_filter_level
        .map(|level| {
            ContentFilterLevel::from_i16(level)
                .map_err(|_| ApiError::validation(format!("Invalid content filter level: {level}")))
        })
        .transpose()?;
    let service = &state.user.user_privacy_service;
    let settings = service
        .update_content_settings(user_id, request.allow_nsfw_content, filter_level)
        .await?;
    Ok(Json(PrivacySettingsResponse::from(settings)))
}

/// POST /@me/privacy/preset
#[utoipa::path(
    post,
    path = "/api/v1/users/@me/privacy/preset",
    request_body = ApplyPresetRequest,
    responses(
        (status = 200, description = "Privacy preset applied", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid JSON body"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn apply_preset(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(request): Json<ApplyPresetRequest>,
) -> Result<Json<PrivacySettingsResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
    let preset = match request.preset {
        PrivacyPresetInput::Public => PrivacyPreset::Public,
        PrivacyPresetInput::FriendsOnly => PrivacyPreset::FriendsOnly,
        PrivacyPresetInput::Private => PrivacyPreset::Private,
    };
    let service = &state.user.user_privacy_service;
    let settings = service.apply_preset(user_id, preset).await?;
    Ok(Json(PrivacySettingsResponse::from(settings)))
}

// ============================================
// RELATIONSHIPS
// ============================================

/// POST /@me/relationships/request
#[utoipa::path(
    post,
    path = "/api/v1/users/@me/relationships/request",
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "Friend request sent", body = RelationshipResponse),
        (status = 400, description = "Invalid JSON"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Blocked or privacy settings prevent request"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn send_friend_request(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// PUT /@me/relationships/request/accept
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/relationships/request/accept",
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "Friend request accepted", body = RelationshipResponse),
        (status = 400, description = "Invalid JSON"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn accept_friend_request(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// PUT /@me/relationships/request/decline
#[utoipa::path(
    put,
    path = "/api/v1/users/@me/relationships/request/decline",
    request_body = RelationshipRequest,
    responses(
        (status = 204, description = "Friend request declined"),
        (status = 400, description = "Invalid JSON"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn decline_friend_request(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(payload): Json<RelationshipRequest>,
) -> Result<StatusCode, ApiError> {
    payload.validate()?;
    let user_id = extract_user_id(&claims)?;
    state
        .user
        .relationship_service
        .decline_friend_request(user_id, payload.target_user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /@me/relationships/friend/:target_user_id
#[utoipa::path(
    delete,
    path = "/api/v1/users/@me/relationships/friend/{target_user_id}",
    params(
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 204, description = "Friend removed"),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn remove_friend(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Path(target_user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let user_id = extract_user_id(&claims)?;
    state
        .user
        .relationship_service
        .remove_friend(user_id, target_user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /@me/relationships/block
#[utoipa::path(
    post,
    path = "/api/v1/users/@me/relationships/block",
    request_body = RelationshipRequest,
    responses(
        (status = 200, description = "User blocked", body = RelationshipResponse),
        (status = 400, description = "Invalid JSON"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn block_user(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(payload): Json<RelationshipRequest>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    payload.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// DELETE /@me/relationships/block/:target_user_id
#[utoipa::path(
    delete,
    path = "/api/v1/users/@me/relationships/block/{target_user_id}",
    params(
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 204, description = "User unblocked"),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn unblock_user(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Path(target_user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let user_id = extract_user_id(&claims)?;
    state
        .user
        .relationship_service
        .unblock_user(user_id, target_user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /@me/relationships/friends
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/relationships/friends",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "Friends list", body = [RelationshipResponse]),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_friends(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// GET /@me/relationships/incoming
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/relationships/incoming",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "Incoming friend requests", body = [RelationshipResponse]),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_incoming_requests(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// GET /@me/relationships/outgoing
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/relationships/outgoing",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "Outgoing friend requests", body = [RelationshipResponse]),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_outgoing_requests(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// GET /@me/relationships/blocked
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/relationships/blocked",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "Blocked users list", body = [RelationshipResponse]),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Validation failed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_blocked_users(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<RelationshipResponse>>, ApiError> {
    pagination.validate()?;
    let user_id = extract_user_id(&claims)?;
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

/// GET /@me/relationships/:target_user_id
#[utoipa::path(
    get,
    path = "/api/v1/users/@me/relationships/{target_user_id}",
    params(
        ("target_user_id" = Uuid, Path, description = "Target User ID")
    ),
    responses(
        (status = 200, description = "Relationship found", body = RelationshipResponse),
        (status = 400, description = "Invalid User ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Relationship not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "user-me"
)]
pub async fn get_relationship(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Path(target_user_id): Path<Uuid>,
) -> Result<Json<RelationshipResponse>, ApiError> {
    let user_id = extract_user_id(&claims)?;
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

// ============================================
// HELPERS
// ============================================

fn extract_user_id(
    claims: &common::infrastructure::security::jwt_manager::Claims,
) -> Result<Uuid, ApiError> {
    claims
        .user_id()
        .map_err(|_| ApiError::internal("Invalid user ID in token"))
}
