use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::domain::user::entity::User;
use crate::domain::user::error::UserDomainError;
use crate::domain::user::valueobject::{CustomStatus, DmPrivacy, FriendRequestPrivacy, UserPrivacySettings, UserStatus};
// =====================================================
// RESPONSE DTOs
// =====================================================

/// Public user profile response — what other users see
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    /// Online status — respects privacy settings
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<CustomStatusDto>,

    /// Role badge (None for regular users, Some("MOD"/"ADMIN") for staff)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_badge: Option<String>,

    pub created_at: DateTime<Utc>,
}

impl UserProfileResponse {
    /// Build response with privacy-aware status visibility
    pub fn from_user(user: &User, viewer_can_see_status: bool) -> Self {
        let visible_status = if viewer_can_see_status {
            user.status.clone()
        } else {
            UserStatus::Offline
        };

        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            banner_url: user.banner_url.clone(),
            bio: user.bio.clone(),
            status: visible_status.as_str().to_string(),
            custom_status: user.custom_status.as_ref().map(CustomStatusDto::from),
            role_badge: user.role.badge().map(|s| s.to_string()),
            created_at: user.created_at,
        }
    }
}

/// User's own profile response — includes private settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProfileResponse {
    #[serde(flatten)]
    pub profile: UserProfileResponse,

    /// Privacy settings (only visible to self)
    pub privacy_settings: PrivacySettingsDto,
}

impl MyProfileResponse {
    pub fn from_user(user: &User) -> Self {
        Self {
            profile: UserProfileResponse::from_user(user, true), // Own status always visible
            privacy_settings: PrivacySettingsDto::from(&user.privacy_settings),
        }
    }
}

/// Custom status DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatusDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<&CustomStatus> for CustomStatusDto {
    fn from(status: &CustomStatus) -> Self {
        Self {
            text: status.text.clone(),
            emoji: status.emoji.clone(),
            expires_at: status.expires_at,
        }
    }
}

/// Privacy settings DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettingsDto {
    pub allow_dms_from: String,             // "everyone" | "friends" | "server_members" | "none"
    pub allow_friend_requests_from: String, // "everyone" | "friends_of_friends" | "none"
    pub show_online_status: bool,
}

impl From<&UserPrivacySettings> for PrivacySettingsDto {
    fn from(settings: &UserPrivacySettings) -> Self {
        Self {
            allow_dms_from: settings.allow_dms_from.as_str().to_string(),
            allow_friend_requests_from: settings.allow_friend_requests_from.as_str().to_string(),
            show_online_status: settings.show_online_status,
        }
    }
}

// =====================================================
// REQUEST DTOs
// =====================================================

/// Update profile request — all fields optional
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

impl UpdateProfileRequest {
    /// Check if request is empty (no fields provided)
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.avatar_url.is_none()
            && self.banner_url.is_none()
            && self.bio.is_none()
    }
}

/// Update privacy settings request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePrivacySettingsRequest {
    pub allow_dms_from: String,             // "everyone" | "friends" | "server_members" | "none"
    pub allow_friend_requests_from: String, // "everyone" | "friends_of_friends" | "none"
    pub show_online_status: bool,
}

impl UpdatePrivacySettingsRequest {
    /// Convert to domain UserPrivacySettings, validating enum values
    pub fn to_domain(&self) -> Result<UserPrivacySettings, UserDomainError> {
        Ok(UserPrivacySettings {
            allow_dms_from: DmPrivacy::from_str(&self.allow_dms_from)?,
            allow_friend_requests_from: FriendRequestPrivacy::from_str(
                &self.allow_friend_requests_from,
            )?,
            show_online_status: self.show_online_status,
        })
    }
}

/// Set custom status request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SetCustomStatusRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,

    /// Optional expiration timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

// =====================================================
// SEARCH DTOs
// =====================================================

/// User search result (minimal public info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_badge: Option<String>,
}

impl From<&User> for UserSearchResult {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            role_badge: user.role.badge().map(|s| s.to_string()),
        }
    }
}