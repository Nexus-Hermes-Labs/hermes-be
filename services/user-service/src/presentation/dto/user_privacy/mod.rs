use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::user_privacy::UserPrivacySettings;

// ============================================
// DM PRIVACY INPUT ENUM
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DmPrivacyInput {
    Everyone,
    Friends,
    ServerMembers,
    None,
}

// ============================================
// UPDATE DM PRIVACY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateDmPrivacyRequest {
    pub allow_dms_from: DmPrivacyInput,
}

// ============================================
// FRIEND REQUEST PRIVACY INPUT ENUM
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestPrivacyInput {
    Everyone,
    FriendsOfFriends,
    None,
}

// ============================================
// UPDATE FRIEND REQUEST PRIVACY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateFriendRequestPrivacyRequest {
    pub allow_friend_requests_from: FriendRequestPrivacyInput,
}

// ============================================
// UPDATE VISIBILITY REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateVisibilityRequest {
    pub show_online_status: Option<bool>,
    pub show_current_activity: Option<bool>,
    pub show_profile_to_non_friends: Option<bool>,
}

// ============================================
// UPDATE CONTENT SETTINGS REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateContentSettingsRequest {
    pub allow_nsfw_content: Option<bool>,
    #[schema(minimum = 0, maximum = 2)]
    pub content_filter_level: Option<i16>, // 0: Off, 1: Medium, 2: Strict
}

// ============================================
// PRIVACY PRESET INPUT ENUM
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyPresetInput {
    Public,
    FriendsOnly,
    Private,
}

// ============================================
// APPLY PRESET REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct ApplyPresetRequest {
    pub preset: PrivacyPresetInput,
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
