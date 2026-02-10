use serde::Deserialize;
use validator::Validate;

// ============================================
// POST /api/auth/login
// Purpose: Authenticate user with email and password
// Auth: Public (no authentication required)
// ============================================

/// Request body for user login
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    /// User's email address
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// User's password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// Optional device name for session tracking
    ///
    /// Example: "Chrome on Windows", "iPhone 14"
    pub device_name: Option<String>,
}

// ============================================
// RESPONSE TYPE ALIAS
// ============================================

/// Response body for successful login
///
/// This is a type alias to `AuthResponse` from the shared module.
/// Returns authentication tokens only (no user profile).
///
/// Note: User profile is not included to keep the response lightweight.
/// Client can call GET /api/users/me to fetch profile if needed.
///
/// Example:
/// ```json
/// {
///   "access_token": "eyJ...",
///   "refresh_token": "eyJ...",
///   "expires_in": 21600,
///   "token_type": "Bearer"
/// }
/// ```
pub use super::shared::AuthResponse as LoginResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_login_request() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            device_name: Some("Chrome".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_valid_login_request_without_device() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            device_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_empty_password() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            device_name: None,
        };

        assert!(request.validate().is_err());
    }
}
