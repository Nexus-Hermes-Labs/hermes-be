use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================
// GET /api/v1/auth/oauth/google
// Purpose: Return the Google consent URL for the SPA to navigate to.
// Auth: Public
// ============================================

/// Response for starting Google authorization.
///
/// The SPA navigates the browser to `authorize_url`; Google then redirects back
/// to the frontend callback route with `code` + `state`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthorizeUrlResponse {
    /// Fully-built Google consent URL (includes CSRF state + PKCE challenge).
    pub authorize_url: String,
}

// ============================================
// POST /api/v1/auth/oauth/google/callback
// Purpose: Exchange the brokered authorization code for Hermes tokens.
// Auth: Public
// ============================================

/// Request body for the Google callback, posted by the frontend after Google
/// redirects to its callback route.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct GoogleCallbackRequest {
    /// Authorization code returned by Google.
    #[validate(length(min = 1, message = "code is required"))]
    pub code: String,

    /// Opaque CSRF state issued at authorization start.
    #[validate(length(min = 1, message = "state is required"))]
    pub state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_callback_request() {
        let request = GoogleCallbackRequest {
            code: "auth-code".to_string(),
            state: "state-token".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_empty_fields_rejected() {
        let request = GoogleCallbackRequest {
            code: String::new(),
            state: String::new(),
        };
        assert!(request.validate().is_err());
    }
}
