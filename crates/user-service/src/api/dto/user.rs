use crate::domain::user::entity::User;
use crate::domain::user::error::UserDomainError;
use crate::domain::user::valueobject::{
    CustomStatus, DmPrivacy, FriendRequestPrivacy, UserPrivacySettings, UserStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// =====================================================
// RESPONSE DTOs - PUBLIC PROFILE
// =====================================================

/// Public user profile response
///
/// What other users see (respects privacy settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,

    /// Online status (respects privacy settings)
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<CustomStatusDto>,

    /// Role badge (None for regular users, Some("MOD"/"ADMIN") for staff)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_badge: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserProfileResponse {
    /// Create response with privacy-aware status visibility
    pub fn new(user: &User, viewer_can_see_status: bool) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            banner_url: user.banner_url.clone(),
            status: Self::get_visible_status(user, viewer_can_see_status),
            custom_status: user.custom_status.as_ref().map(CustomStatusDto::from),
            role_badge: user.role.badge().map(String::from),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    /// Create response for public viewing (friend/stranger)
    pub fn public(user: &User, viewer_can_see_status: bool) -> Self {
        Self::new(user, viewer_can_see_status)
    }

    /// Create response for self viewing
    pub fn for_self(user: &User) -> Self {
        Self::new(user, true) // Always show own status
    }

    /// Get visible status based on privacy settings
    fn get_visible_status(user: &User, can_see: bool) -> String {
        if can_see {
            user.status.as_str().to_string()
        } else {
            UserStatus::Offline.as_str().to_string()
        }
    }
}

// =====================================================
// RESPONSE DTOs - PRIVATE PROFILE
// =====================================================

/// User's own profile response
///
/// Includes private settings (privacy, email, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProfileResponse {
    // Public profile fields (flattened)
    #[serde(flatten)]
    pub profile: UserProfileResponse,

    // Email (only visible to self)
    pub email: String,

    // Privacy settings (only visible to self)
    pub privacy_settings: PrivacySettingsDto,
}

impl MyProfileResponse {
    pub fn new(user: &User) -> Self {
        Self {
            profile: UserProfileResponse::for_self(user),
            email: user.email.clone(),
            privacy_settings: PrivacySettingsDto::from(&user.privacy_settings),
        }
    }
}

impl From<&User> for MyProfileResponse {
    fn from(user: &User) -> Self {
        Self::new(user)
    }
}

// =====================================================
// RESPONSE DTOs - UPDATE RESPONSE
// =====================================================

/// Profile update response
///
/// Returns updated profile fields after PATCH /users/me
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,

    pub updated_at: DateTime<Utc>,
}

impl From<&User> for UserUpdateResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            banner_url: user.banner_url.clone(),
            updated_at: user.updated_at,
        }
    }
}

// =====================================================
// RESPONSE DTOs - SEARCH
// =====================================================

/// User search result
///
/// Minimal public info for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    pub status: String,
}

impl From<&User> for UserSearchResult {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            status: user.status.as_str().to_string(),
        }
    }
}

// =====================================================
// NESTED DTOs - CUSTOM STATUS
// =====================================================

/// Custom status DTO
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl From<CustomStatus> for CustomStatusDto {
    fn from(status: CustomStatus) -> Self {
        Self::from(&status)
    }
}

// =====================================================
// NESTED DTOs - PRIVACY SETTINGS
// =====================================================

/// Privacy settings DTO
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacySettingsDto {
    /// Who can send DMs: "everyone" | "friends" | "server_members" | "none"
    pub allow_dms_from: String,

    /// Who can send friend requests: "everyone" | "friends_of_friends" | "none"
    pub allow_friend_requests_from: String,

    /// Whether to show online status
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

impl From<UserPrivacySettings> for PrivacySettingsDto {
    fn from(settings: UserPrivacySettings) -> Self {
        Self::from(&settings)
    }
}

// =====================================================
// REQUEST DTOs - PROFILE UPDATE
// =====================================================

/// Update profile request
///
/// PATCH /v1/users/me - all fields optional
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 32, message = "Display name must be 1-32 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[validate(url(message = "Avatar URL must be valid HTTPS URL"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[validate(url(message = "Banner URL must be valid HTTPS URL"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,

    #[validate(length(max = 190, message = "Bio must be at most 190 characters"))]
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

    /// Validate and sanitize fields
    pub fn sanitize(mut self) -> Self {
        // Trim whitespace from text fields
        if let Some(ref mut display_name) = self.display_name {
            *display_name = display_name.trim().to_string();
        }

        if let Some(ref mut bio) = self.bio {
            *bio = bio.trim().to_string();
        }

        self
    }
}

// =====================================================
// REQUEST DTOs - PRIVACY UPDATE
// =====================================================

/// Update privacy settings request
///
/// PATCH /v1/users/me/privacy
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePrivacySettingsRequest {
    /// Who can send DMs: "everyone" | "friends" | "server_members" | "none"
    #[validate(length(min = 1), custom(function = "validate_dm_privacy"))]
    pub allow_dms_from: String,

    /// Who can send friend requests: "everyone" | "friends_of_friends" | "none"
    #[validate(length(min = 1), custom(function = "validate_friend_request_privacy"))]
    pub allow_friend_requests_from: String,

    /// Whether to show online status
    pub show_online_status: bool,
}

impl UpdatePrivacySettingsRequest {
    /// Convert to domain UserPrivacySettings (validates enum values)
    pub fn to_domain(self) -> Result<UserPrivacySettings, UserDomainError> {
        Ok(UserPrivacySettings {
            allow_dms_from: DmPrivacy::from_str(&self.allow_dms_from)?,
            allow_friend_requests_from: FriendRequestPrivacy::from_str(
                &self.allow_friend_requests_from,
            )?,
            show_online_status: self.show_online_status,
        })
    }
}

// =====================================================
// REQUEST DTOs - CUSTOM STATUS
// =====================================================

/// Set custom status request
///
/// PUT /v1/users/me/status
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SetCustomStatusRequest {
    #[validate(length(max = 128, message = "Status text must be at most 128 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,

    /// Optional expiration timestamp (ISO 8601, max 24 hours from now)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl SetCustomStatusRequest {
    /// Convert to domain CustomStatus
    pub fn to_domain(self) -> Result<CustomStatus, UserDomainError> {
        // Validate expiration is not in the past
        if let Some(expires_at) = self.expires_at {
            if expires_at <= Utc::now() {
                return Err(UserDomainError::CustomStatusExpirationInPast);
            }

            // TODO: Max 24 hours from now
            // let max_expiry = Utc::now() + chrono::Duration::hours(24);
            // if expires_at > max_expiry {
            //     return Err(UserDomainError::InvalidCustomStatus(
            //         "Expiration cannot be more than 24 hours from now".to_string(),
            //     ));
            // }
        }

        Ok(CustomStatus {
            text: self.text,
            emoji: self.emoji,
            expires_at: self.expires_at,
        })
    }

    /// Sanitize text fields
    pub fn sanitize(mut self) -> Self {
        if let Some(ref mut text) = self.text {
            *text = text.trim().to_string();
        }
        self
    }
}

// =====================================================
// VALIDATION HELPERS
// =====================================================

/// Validate DM privacy enum value
fn validate_dm_privacy(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "everyone" | "friends" | "server_members" | "none" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_dm_privacy")),
    }
}

/// Validate friend request privacy enum value
fn validate_friend_request_privacy(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "everyone" | "friends_of_friends" | "none" => Ok(()),
        _ => Err(validator::ValidationError::new(
            "invalid_friend_request_privacy",
        )),
    }
}

// =====================================================
// TESTS
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::UserRole;

    #[test]
    fn test_update_profile_request_is_empty() {
        let empty = UpdateProfileRequest {
            display_name: None,
            avatar_url: None,
            banner_url: None,
            bio: None,
        };
        assert!(empty.is_empty());

        let not_empty = UpdateProfileRequest {
            display_name: Some("Alice".to_string()),
            avatar_url: None,
            banner_url: None,
            bio: None,
        };
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_update_profile_request_sanitize() {
        let request = UpdateProfileRequest {
            display_name: Some("  Alice  ".to_string()),
            avatar_url: None,
            banner_url: None,
            bio: Some("  Cool bio  ".to_string()),
        };

        let sanitized = request.sanitize();
        assert_eq!(sanitized.display_name, Some("Alice".to_string()));
        assert_eq!(sanitized.bio, Some("Cool bio".to_string()));
    }

    #[test]
    fn test_custom_status_dto_from_domain() {
        let domain = CustomStatus {
            text: Some("Working".to_string()),
            emoji: Some("💼".to_string()),
            expires_at: None,
        };

        let dto = CustomStatusDto::from(&domain);
        assert_eq!(dto.text, Some("Working".to_string()));
        assert_eq!(dto.emoji, Some("💼".to_string()));
        assert_eq!(dto.expires_at, None);
    }

    #[test]
    fn test_privacy_settings_dto_from_domain() {
        let domain = UserPrivacySettings {
            allow_dms_from: DmPrivacy::Friends,
            allow_friend_requests_from: FriendRequestPrivacy::FriendsOfFriends,
            show_online_status: true,
        };

        let dto = PrivacySettingsDto::from(&domain);
        assert_eq!(dto.allow_dms_from, "friends");
        assert_eq!(dto.allow_friend_requests_from, "friends_of_friends");
        assert_eq!(dto.show_online_status, true);
    }

    #[test]
    fn test_set_custom_status_request_to_domain_invalid_expiry() {
        // Past expiry
        let request = SetCustomStatusRequest {
            text: Some("Test".to_string()),
            emoji: None,
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
        };

        assert!(request.to_domain().is_err());

        // Too far in future (> 24 hours)
        let request = SetCustomStatusRequest {
            text: Some("Test".to_string()),
            emoji: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(25)),
        };

        assert!(request.to_domain().is_err());
    }

    #[test]
    fn test_set_custom_status_request_sanitize() {
        let request = SetCustomStatusRequest {
            text: Some("  Working on project  ".to_string()),
            emoji: Some("💻".to_string()),
            expires_at: None,
        };

        let sanitized = request.sanitize();
        assert_eq!(sanitized.text, Some("Working on project".to_string()));
    }

    #[test]
    fn test_user_profile_response_privacy() {
        let user = User {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            discriminator: "0001".to_string(),
            email: "alice@example.com".to_string(),
            display_name: "Alice".to_string(),
            bio: None,
            avatar_url: None,
            banner_url: None,
            status: UserStatus::Online,
            custom_status: None,
            privacy_settings: UserPrivacySettings::default(),
            role: UserRole::User,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Viewer can see status
        let response = UserProfileResponse::public(&user, true);
        assert_eq!(response.status, "online");

        // Viewer cannot see status (privacy)
        let response = UserProfileResponse::public(&user, false);
        assert_eq!(response.status, "offline");

        // Self can always see status
        let response = UserProfileResponse::for_self(&user);
        assert_eq!(response.status, "online");
    }
}
