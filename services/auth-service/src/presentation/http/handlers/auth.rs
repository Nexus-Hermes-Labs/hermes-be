use crate::application::services::authentication::error::AuthApplicationError;
use crate::presentation::http::dto::{
    AuthResponse, AuthResponseWithUser, ClientInfo, LoginRequest, LogoutRequest, LogoutResponse,
    RefreshTokenRequest, RegisterRequest, VerifyEmailRequest, VerifyEmailResponse,
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = AuthResponseWithUser),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    tag = "auth"
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Account locked/suspended")
    ),
    tag = "auth"
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid/expired refresh token")
    ),
    tag = "auth"
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Token(s) revoked successfully", body = LogoutResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid refresh token")
    ),
    tag = "auth"
)]
pub async fn logout_handler(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Response, ApiError> {
    request.validate()?;

    let claims = state
        .auth
        .jwt_manager
        .verify_refresh_token(&request.refresh_token)
        .map_err(|_e| Into::<ApiError>::into(AuthApplicationError::InvalidToken))?;

    let user_id = claims
        .user_id()
        .map_err(|_e| Into::<ApiError>::into(AuthApplicationError::InvalidToken))?;

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
#[utoipa::path(
    get,
    path = "/api/v1/auth/verify-email",
    params(
        VerifyEmailRequest
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid or expired token")
    ),
    tag = "auth"
)]
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
