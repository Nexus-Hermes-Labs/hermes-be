use serde::{Deserialize, Serialize};
use validator::Validate;
use lazy_static::lazy_static;
// ============================================
// CREATE PROFILE REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProfileRequest {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    #[validate(regex(path = "USERNAME_REGEX", message = "Username can only contain lowercase letters, numbers, and underscores"))]
    pub username: String,

    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,
}

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9_]+$").unwrap();
}

// ============================================
// UPDATE PROFILE REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: Option<String>,

    #[validate(length(max = 500, message = "Bio must be max 500 characters"))]
    pub bio: Option<String>,

    #[validate(url(message = "Avatar URL must be valid"))]
    pub avatar_url: Option<String>,

    #[validate(url(message = "Banner URL must be valid"))]
    pub banner_url: Option<String>,
}

// ============================================
// CHANGE USERNAME REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeUsernameRequest {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    #[validate(regex(path = "USERNAME_REGEX", message = "Username can only contain lowercase letters, numbers, and underscores"))]
    pub new_username: String,
}

// ============================================
// UPDATE STATUS REQUEST
// ============================================

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String, // "online", "offline", "idle", "dnd"
}

// ============================================
// SET CUSTOM STATUS REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct SetCustomStatusRequest {
    #[validate(length(max = 128, message = "Custom status text must be max 128 characters"))]
    pub text: Option<String>,

    #[validate(length(max = 50, message = "Custom status emoji must be max 50 characters"))]
    pub emoji: Option<String>,

    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================
// SEARCH USERS REQUEST
// ============================================

#[derive(Debug, Deserialize)]
pub struct SearchUsersRequest {
    pub query: String,

    #[serde(default = "default_limit")]
    pub limit: i64,

    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    10
}

// ============================================
// CHECK USERNAME AVAILABILITY REQUEST
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct CheckUsernameRequest {
    #[validate(length(min = 3, max = 32))]
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
