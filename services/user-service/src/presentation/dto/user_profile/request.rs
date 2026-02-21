use once_cell::sync::Lazy;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

// ============================================
// CREATE PROFILE REQUEST
// ============================================

static USERNAME_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^[a-z0-9_]+$").unwrap());

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProfileRequest {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    #[validate(regex(
        path = "*USERNAME_REGEX",
        message = "Username can only contain lowercase letters, numbers, and underscores"
    ))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[schema(min_length = 3, max_length = 32, pattern = "^[a-z0-9_]+$")]
    pub username: String,

    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[validate(custom(function = "common::utils::validator::validate_no_control_chars"))]
    #[schema(min_length = 1, max_length = 100)]
    pub display_name: String,
}

// ============================================
// UPDATE PROFILE REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[validate(custom(function = "common::utils::validator::validate_no_control_chars"))]
    #[schema(
        min_length = 1,
        max_length = 100,
        pattern = r"^[^\x00-\x1f\x80-\x9f]+$"
    )]
    pub display_name: Option<String>,

    #[validate(length(max = 500, message = "Bio must be max 500 characters"))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    pub bio: Option<String>,

    #[validate(url(message = "Avatar URL must be valid"))]
    #[schema(min_length = 1, format = "uri")]
    pub avatar_url: Option<String>,

    #[validate(url(message = "Banner URL must be valid"))]
    #[schema(min_length = 1, format = "uri")]
    pub banner_url: Option<String>,
}

// ============================================
// CHANGE USERNAME REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangeUsernameRequest {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    #[validate(regex(
        path = "*USERNAME_REGEX",
        message = "Username can only contain lowercase letters, numbers, and underscores"
    ))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[schema(min_length = 3, max_length = 32, pattern = "^[a-z0-9_]+$")]
    pub new_username: String,
}

// ============================================
// USER STATUS INPUT ENUM
// ============================================

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserStatusInput {
    Online,
    Offline,
    Idle,
    Dnd,
}

// ============================================
// UPDATE STATUS REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateStatusRequest {
    pub status: UserStatusInput,
}

// ============================================
// SET CUSTOM STATUS REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SetCustomStatusRequest {
    #[validate(length(max = 128, message = "Custom status text must be max 128 characters"))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    pub text: Option<String>,

    #[validate(length(max = 50, message = "Custom status emoji must be max 50 characters"))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    pub emoji: Option<String>,

    #[schema(min_length = 20)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================
// SEARCH USERS REQUEST
// ============================================

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams, Validate)]
#[serde(deny_unknown_fields)]
pub struct SearchUsersRequest {
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[schema(min_length = 1)]
    pub query: String,

    #[serde(default = "default_limit_search")]
    #[validate(range(min = 1, max = 100))]
    #[schema(minimum = 1, maximum = 100, format = "int64")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: i64,

    #[serde(default)]
    #[validate(range(min = 0, max = 1_000_000_000))]
    #[schema(minimum = 0, maximum = 1_000_000_000, format = "int64")]
    #[param(minimum = 0, maximum = 1_000_000_000)]
    pub offset: i64,
}

fn default_limit_search() -> i64 {
    10
}

// ============================================
// CHECK USERNAME AVAILABILITY REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckUsernameRequest {
    #[validate(length(min = 3, max = 32))]
    #[validate(regex(
        path = "*USERNAME_REGEX",
        message = "Username can only contain lowercase letters, numbers, and underscores"
    ))]
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    #[schema(min_length = 3, max_length = 32, pattern = "^[a-z0-9_]+$")]
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_profile_validation() {
        let valid = CreateProfileRequest {
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid username (too short)
        let invalid = CreateProfileRequest {
            username: "ab".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(invalid.validate().is_err());

        // Invalid username (uppercase)
        let invalid = CreateProfileRequest {
            username: "TestUser".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_update_profile_validation() {
        let valid = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            bio: Some("New bio".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            banner_url: None,
        };
        assert!(valid.validate().is_ok());

        // Invalid bio (too long)
        let invalid = UpdateProfileRequest {
            display_name: None,
            bio: Some("a".repeat(501)),
            avatar_url: None,
            banner_url: None,
        };
        assert!(invalid.validate().is_err());
    }
}
