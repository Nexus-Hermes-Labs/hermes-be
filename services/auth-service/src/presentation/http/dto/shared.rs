use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_client_ip::SecureClientIp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================
// BASE AUTHENTICATION RESPONSE TYPES
// ============================================

/// Standard authentication response (tokens only)
///
/// Used by:
/// - POST /auth/login (as LoginResponse)
/// - POST /auth/refresh (as RefreshTokenResponse)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token (short-lived, ~6 hours)
    pub access_token: String,

    /// JWT refresh token (long-lived, ~30 days)
    pub refresh_token: String,

    /// Token expiration time in seconds
    pub expires_in: i64,

    /// Token type (always "Bearer")
    pub token_type: String,
}

impl AuthResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}

/// Authentication response with user_profile profile
///
/// Used by:
/// - POST /auth/register (as RegisterResponse)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponseWithUser {
    /// JWT access token (short-lived, ~6 hours)
    pub access_token: String,

    /// JWT refresh token (long-lived, ~30 days)
    pub refresh_token: String,

    /// Token expiration time in seconds
    pub expires_in: i64,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// User profile information
    pub user: UserProfile,
}

impl AuthResponseWithUser {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user: UserProfile,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            token_type: "Bearer".to_string(),
            user,
        }
    }

    /// Convert to base AuthResponse (without user_profile)
    pub fn into_auth_response(self) -> AuthResponse {
        AuthResponse {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_in: self.expires_in,
            token_type: self.token_type,
        }
    }
}

// ============================================
// USER PROFILE (from user_profile-service via gRPC)
// ============================================

/// User profile information
///
/// This is returned from user_profile-service via gRPC and included
/// in registration responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    /// User's unique identifier (UUID)
    pub user_id: Uuid,

    /// User's email address
    pub email: String,

    /// User's username
    pub username: String,

    /// User's display name
    ///
    /// Example: "John Doe"
    pub display_name: String,

    /// Avatar URL (optional)
    pub avatar: Option<String>,

    /// User bio (optional)
    pub bio: Option<String>,

    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================
// CLIENT INFO (from HTTP request)
// ============================================

/// Client information extracted from HTTP request
///
/// Used for session tracking and audit logging.
#[derive(Debug, Clone, Default)]
pub struct ClientInfo {
    /// Client IP address
    ///
    /// Extracted from:
    /// 1. X-Forwarded-For header (if behind proxy)
    /// 2. X-Real-IP header
    /// 3. Connection remote address
    pub ip_address: Option<String>,

    /// User agent string
    ///
    /// Example: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)..."
    pub user_agent: Option<String>,

    /// Device name (from request)
    ///
    /// Optional, provided by client in login request.
    pub device_name: Option<String>,

    /// Stable per-device identifier supplied by the client via the
    /// `X-Device-Id` header.
    ///
    /// Used to enforce one active session per device on login: if a session
    /// already exists for `(credential, device_id)`, it is revoked before a
    /// new one is created. Sessions without a `device_id` (older clients) are
    /// allowed to coexist as before.
    pub device_id: Option<String>,
}

impl ClientInfo {
    pub fn new(
        ip_address: Option<String>,
        user_agent: Option<String>,
        device_name: Option<String>,
        device_id: Option<String>,
    ) -> Self {
        Self {
            ip_address,
            user_agent,
            device_name,
            device_id,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ClientInfo
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Resolve client IP via the source configured on the router (typically
        // RightmostXForwardedFor when running behind Traefik). Missing source
        // or unparseable header simply yields no IP — never reject the request.
        let ip_address = SecureClientIp::from_request_parts(parts, state)
            .await
            .ok()
            .map(|SecureClientIp(addr)| addr.to_string());

        let user_agent = parts
            .headers
            .get("user-agent")
            .and_then(|h_val| h_val.to_str().ok())
            .map(|s| s.to_string());

        let device_id = parts
            .headers
            .get("x-device-id")
            .and_then(|h_val| h_val.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(ClientInfo::new(ip_address, user_agent, None, device_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_creation() {
        let response = AuthResponse::new("access".to_string(), "refresh".to_string(), 21600);

        assert_eq!(response.access_token, "access");
        assert_eq!(response.refresh_token, "refresh");
        assert_eq!(response.expires_in, 21600);
        assert_eq!(response.token_type, "Bearer");
    }

    #[test]
    fn test_auth_response_with_user_creation() {
        let user = UserProfile {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),

            display_name: "Test User".to_string(),
            avatar: None,
            bio: None,
            created_at: Utc::now(),
        };

        let response = AuthResponseWithUser::new(
            "access".to_string(),
            "refresh".to_string(),
            21600,
            user.clone(),
        );

        assert_eq!(response.access_token, "access");
        assert_eq!(response.user.email, "test@example.com");
    }

    #[test]
    fn test_convert_to_base_response() {
        let user = UserProfile {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),

            display_name: "Test User".to_string(),
            avatar: None,
            bio: None,
            created_at: Utc::now(),
        };

        let response_with_user =
            AuthResponseWithUser::new("access".to_string(), "refresh".to_string(), 21600, user);

        let base_response = response_with_user.into_auth_response();

        assert_eq!(base_response.access_token, "access");
        assert_eq!(base_response.refresh_token, "refresh");
    }

    #[test]
    fn test_client_info_default() {
        let info = ClientInfo::default();

        assert!(info.ip_address.is_none());
        assert!(info.user_agent.is_none());
        assert!(info.device_name.is_none());
    }

    #[test]
    fn test_client_info_new() {
        let info = ClientInfo::new(
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            Some("Chrome".to_string()),
            Some("device-abc".to_string()),
        );

        assert_eq!(info.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(info.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(info.device_name, Some("Chrome".to_string()));
        assert_eq!(info.device_id, Some("device-abc".to_string()));
    }
}
