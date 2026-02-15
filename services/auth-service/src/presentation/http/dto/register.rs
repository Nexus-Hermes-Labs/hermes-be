use std::sync::LazyLock;
use serde::Deserialize;
use validator::Validate;

// ============================================
// POST /api/auth/register
// Purpose: Register a new user_profile account
// Auth: Public (no authentication required)
// ============================================

/// Request body for user_profile registration
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    /// User's email address (must be unique)
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// Unique username (3-32 characters, lowercase alphanumeric + underscore)
    #[validate(length(
        min = 3,
        max = 32,
        message = "Username must be between 3 and 32 characters"
    ))]
    #[validate(regex(
        path = *USERNAME_REGEX,
        message = "Username can only contain lowercase letters, numbers, and underscores"
    ))]
    pub username: String,

    /// User's display name (1-100 characters)
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,

    /// Password (minimum 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

// ============================================
// RESPONSE TYPE ALIAS
// ============================================

/// Response body for successful registration
///
/// This is a type alias to `AuthResponseWithUser` from the shared module.
/// Returns authentication tokens AND complete user_profile profile.
///
/// Example:
/// ```json
/// {
///   "access_token": "eyJ...",
///   "refresh_token": "eyJ...",
///   "expires_in": 21600,
///   "token_type": "Bearer",
///   "user_profile": {
///     "user_id": "550e8400-e29b-41d4-a716-446655440000",
///     "email": "user_profile@example.com",
///     "username": "johndoe",
///     "display_name": "John Doe"
///   }
/// }
/// ```
pub use super::shared::AuthResponseWithUser as RegisterResponse;

// ============================================
// VALIDATION REGEX
// ============================================

use regex::Regex;

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9_]+$").expect("Failed to compile username regex")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_register_request() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_short_username() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "ab".to_string(),
            display_name: "Test User".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_invalid_username_uppercase() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "TestUser".to_string(),
            display_name: "Test User".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_invalid_username_spaces() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "test user_profile".to_string(),
            display_name: "Test User".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_short_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "short".to_string(),
        };

        assert!(request.validate().is_err());
    }
}
