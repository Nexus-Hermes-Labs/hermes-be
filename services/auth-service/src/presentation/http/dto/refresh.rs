use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

// ============================================
// POST /api/auth/refresh
// Purpose: Refresh access token using refresh token
// Auth: Public (refresh token in request body)
// Standard: OAuth 2.0 Token Endpoint (RFC 6749)
// ============================================

/// Request body for token refresh
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RefreshTokenRequest {
    /// Valid refresh token received from login/register
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

// ============================================
// RESPONSE TYPE ALIAS
// ============================================

/// Response body for token refresh
///
/// This is a type alias to `AuthResponse` from the shared module.
/// Returns NEW access token and NEW refresh token (token rotation).
///
/// Note: Implements refresh token rotation for security.
/// The old refresh token is invalidated and a new one is issued.
/// Client MUST use the new refresh token for subsequent requests.
///
/// Example:
/// ```json
/// {
///   "access_token": "eyJ...",     // NEW
///   "refresh_token": "eyJ...",    // NEW (old one invalidated)
///   "expires_in": 21600,
///   "token_type": "Bearer"
/// }
/// ```
pub use super::shared::AuthResponse as RefreshTokenResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_refresh_request() {
        let request = RefreshTokenRequest {
            refresh_token: "valid_refresh_token".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_empty_refresh_token() {
        let request = RefreshTokenRequest {
            refresh_token: "".to_string(),
        };

        assert!(request.validate().is_err());
    }
}
