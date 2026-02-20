use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::user_privacy::UserPrivacySettings;

// ============================================
// UPDATE DM PRIVACY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDmPrivacyRequest {
    pub allow_dms_from: String, // "everyone", "friends", "server_members", "none"
}

// ============================================
// UPDATE FRIEND REQUEST PRIVACY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFriendRequestPrivacyRequest {
    pub allow_friend_requests_from: String, // "everyone", "friends_of_friends", "none"
}

// ============================================
// UPDATE VISIBILITY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateVisibilityRequest {
    pub show_online_status: Option<bool>,
    pub show_current_activity: Option<bool>,
    pub show_profile_to_non_friends: Option<bool>,
}

// ============================================
// UPDATE CONTENT SETTINGS REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateContentSettingsRequest {
    pub allow_nsfw_content: Option<bool>,
    pub content_filter_level: Option<i16>, // 0: Off, 1: Medium, 2: Strict
}

// ============================================
// APPLY PRESET REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyPresetRequest {
    pub preset: String, // "public", "friends_only", "private"
}

// ============================================
// PRIVACY SETTINGS RESPONSE
// ============================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PrivacySettingsResponse {
    pub allow_dms_from: String,
    pub allow_friend_requests_from: String,
    pub show_online_status: bool,
    pub show_current_activity: bool,
    pub show_profile_to_non_friends: bool,
    pub allow_nsfw_content: bool,
    pub content_filter_level: i16,
}

impl From<UserPrivacySettings> for PrivacySettingsResponse {
    fn from(settings: UserPrivacySettings) -> Self {
        Self {
            allow_dms_from: settings.allow_dms_from().as_str().to_string(),
            allow_friend_requests_from: settings.allow_friend_requests_from().as_str().to_string(),
            show_online_status: settings.show_online_status(),
            show_current_activity: settings.show_current_activity(),
            show_profile_to_non_friends: settings.show_profile_to_non_friends(),
            allow_nsfw_content: settings.allow_nsfw_content(),
            content_filter_level: settings.content_filter_level().as_i16(),
        }
    }
}
