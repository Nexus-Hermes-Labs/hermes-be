use crate::presentation::http::dto::{
    ClientInfo, LoginRequest, LogoutRequest, RefreshTokenRequest, RegisterRequest,
    VerifyEmailRequest,
};
use crate::presentation::http::error::ApiError;
use crate::application::services::authentication::error::AuthApplicationError;
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
    client_info: ClientInfo,
    Json(request_body): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    request_body.validate()?;

    // Call auth service
    let response = state
        .auth
        .service
        .register(request_body, client_info)
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
    client_info: ClientInfo,
    Json(request_body): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    request_body.validate()?;

    // Call auth service
    let response = state
        .auth
        .service
        .login(request_body, client_info)
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
    request.validate()?;

    let claims = state
        .auth
        .jwt_manager
        .verify_refresh_token(&request.refresh_token)
        .map_err(|_e| {
            Into::<ApiError>::into(AuthApplicationError::InvalidToken)
        })?;

    let user_id = claims.user_id().map_err(|_e| {
                    Into::<ApiError>::into(AuthApplicationError::InvalidToken)    })?;

    // Session ID is optional in the claims, but if present, we use it for specific logout
    // Otherwise, all_devices will handle revoking all sessions for the user.
    let session_id = claims.jwt_id().ok(); // Convert Option<&Uuid> to Option<Uuid>

    let response = state
        .auth
        .service
        .logout(user_id, session_id, request.all_devices)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// GET /api/auth/verify-email?token={token}
///
/// Verify user's email address.
///
/// # Query Parameters
/// - `token` (string, required): The verification token sent to the user's email.
///
/// # Response
/// - `200 OK` - Email verified successfully
/// - `400 Bad Request` - Validation error (e.g., missing token)
/// - `401 Unauthorized` - Invalid or expired token
/// - `500 Internal Server Error` - Server error
pub async fn verify_email_handler(
    State(state): State<AppState>,
    axum::extract::Query(request): axum::extract::Query<VerifyEmailRequest>,
) -> Result<Response, ApiError> {
    request.validate()?;

    state
        .auth
        .service
        .verify_email(&request.token)
        .await
        .map_err(ApiError::from)?;

    let response = crate::presentation::http::dto::VerifyEmailResponse {
        message: "Email verified successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}
