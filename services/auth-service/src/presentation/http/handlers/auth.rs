use std::sync::Arc;

use crate::presentation::http::dto::{
    ClientInfo, LoginRequest, LogoutRequest, RefreshTokenRequest, RegisterRequest,
};
use crate::presentation::http::error::ApiError;
use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use validator::Validate;
// ============================================
// AUTH HANDLERS
// ============================================

/// POST /api/auth/register
///
/// Register a new user_profile account.
///
/// # Request Body
/// ```json
/// {
///   "email": "user_profile@example.com",
///   "username": "johndoe",
///   "display_name": "John Doe",
///   "password": "SecurePass123!"
/// }
/// ```
///
/// # Response
/// - `201 Created` - Registration successful (returns tokens + user_profile profile)
/// - `400 Bad Request` - Validation error
/// - `409 Conflict` - Email already exists
/// - `500 Internal Server Error` - Server error
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    request.validate()?;

    // TODO: Extract client info from HTTP request
    // For now, use default (will be implemented with full HTTP context)
    let client_info = self::extract_client_info_from_request();

    // Call auth service
    let response = state
        .auth
        .service
        .register(request, client_info)
        .await
        .map_err(ApiError::from)?;

    // Return 201 Created with tokens + user_profile profile
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// POST /api/auth/login
///
/// Authenticate user_profile with email and password.
///
/// # Request Body
/// ```json
/// {
///   "email": "user_profile@example.com",
///   "password": "SecurePass123!",
///   "device_name": "Chrome on Windows"  // optional
/// }
/// ```
///
/// # Response
/// - `200 OK` - Login successful (returns tokens only)
/// - `400 Bad Request` - Validation error
/// - `401 Unauthorized` - Invalid credentials
/// - `403 Forbidden` - Account locked/suspended
/// - `500 Internal Server Error` - Server error
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    request.validate()?;

    // TODO: Extract client info from HTTP request
    let client_info = self::extract_client_info_from_request();

    // Call auth service
    let response = state
        .auth
        .service
        .login(request, client_info)
        .await
        .map_err(ApiError::from)?;

    // Return 200 OK with tokens
    Ok((StatusCode::OK, Json(response)).into_response())
}

/// POST /api/auth/refresh
///
/// Refresh access token using refresh token.
///
/// # Request Body
/// ```json
/// {
///   "refresh_token": "eyJ..."
/// }
/// ```
///
/// # Response
/// - `200 OK` - Token refreshed (returns new tokens)
/// - `400 Bad Request` - Validation error
/// - `401 Unauthorized` - Invalid/expired refresh token
/// - `500 Internal Server Error` - Server error
///
/// # Token Rotation
/// The old refresh token is invalidated and a new one is issued.
/// Client MUST use the new refresh token for subsequent requests.
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    request.validate()?;

    // Call auth service
    let response = state
        .auth
        .service
        .refresh_token(request)
        .await
        .map_err(ApiError::from)?;

    // Return 200 OK with new tokens
    Ok((StatusCode::OK, Json(response)).into_response())
}

/// POST /api/auth/logout
///
/// Logout user_profile by revoking refresh token(s).
///
/// Public endpoint following OAuth 2.0 Token Revocation (RFC 7009).
/// No access token required - validates refresh token instead.
///
/// # Request Body
/// ```json
/// {
///   "refresh_token": "eyJ...",
///   "all_devices": false  // true to logout from all devices
/// }
/// ```
///
/// # Response
/// - `200 OK` - Token(s) revoked successfully
/// - `400 Bad Request` - Validation error
/// - `401 Unauthorized` - Invalid refresh token
/// - `500 Internal Server Error` - Server error
pub async fn logout_handler(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Response, ApiError> {
    unimplemented!("Extract user_id and session_id from refresh token");
    // Validate request
    request.validate()?;

    // Extract user_id and session_id from refresh token
    // For MVP, we'll just use the token to find the session
    // TODO: Parse JWT to get user_id and session_id
}

// ============================================
// HELPER: EXTRACT CLIENT INFO FROM REQUEST
// ============================================

/// Extract client information from HTTP request
///
/// This should be implemented as an Axum extractor in production.
/// For now, it's a placeholder showing what needs to be extracted.
///
/// # Example (Future Implementation)
/// ```rust
/// #[async_trait]
/// impl<S> FromRequestParts<S> for ClientInfo {
///     async fn from_request_parts(
///         parts: &mut Parts,
///         _state: &S,
///     ) -> Result<Self, Self::Rejection> {
///         let ip = extract_ip_from_headers(&parts.headers);
///         let user_agent = extract_user_agent(&parts.headers);
///         Ok(ClientInfo::new(ip, user_agent, None))
///     }
/// }
/// ```
#[allow(dead_code)]
fn extract_client_info_from_request(/* req: &Request */) -> ClientInfo {
    // TODO: Implement with full HTTP context
    // Extract from headers:
    // - X-Forwarded-For or X-Real-IP for IP
    // - User-Agent header
    ClientInfo::default()
}
