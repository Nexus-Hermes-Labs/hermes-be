use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================
// POST /api/auth/logout
// Purpose: Revoke refresh token and end session
// Auth: Public (refresh token in request body)
// Standard: OAuth 2.0 Token Revocation (RFC 7009)
// ============================================

/// Request body for logout
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LogoutRequest {
    /// Refresh token to revoke (required)
    #[validate(length(min = 1, message = "Refresh token is required"))]
    #[schema(min_length = 1)]
    pub refresh_token: String,

    /// If true, revoke ALL user_profile sessions (logout from all devices)
    ///
    /// Default: false (only logout from current device)
    #[serde(default)]
    pub all_devices: bool,
}

/// Response body for successful logout
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LogoutResponse {
    /// Success message
    pub message: String,

    /// Number of sessions revoked
    pub sessions_revoked: usize,
}

impl LogoutResponse {
    pub fn single_device() -> Self {
        Self {
            message: "Logged out successfully".to_string(),
            sessions_revoked: 1,
        }
    }

    pub fn all_devices(count: usize) -> Self {
        Self {
            message: format!("Logged out from {} device(s)", count),
            sessions_revoked: count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_logout_request() {
        let request = LogoutRequest {
            refresh_token: "valid_token".to_string(),
            all_devices: false,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_logout_all_devices() {
        let request = LogoutRequest {
            refresh_token: "valid_token".to_string(),
            all_devices: true,
        };

        assert!(request.validate().is_ok());
        assert!(request.all_devices);
    }

    #[test]
    fn test_empty_refresh_token() {
        let request = LogoutRequest {
            refresh_token: "".to_string(),
            all_devices: false,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_default_all_devices() {
        let json = r#"{"refresh_token":"token"}"#;
        let request: LogoutRequest = serde_json::from_str(json).unwrap();
        assert!(!request.all_devices);
    }

    #[test]
    fn test_logout_response_single() {
        let response = LogoutResponse::single_device();
        assert_eq!(response.sessions_revoked, 1);
    }

    #[test]
    fn test_logout_response_all() {
        let response = LogoutResponse::all_devices(3);
        assert_eq!(response.sessions_revoked, 3);
        assert_eq!(response.message, "Logged out from 3 device(s)");
    }
}
