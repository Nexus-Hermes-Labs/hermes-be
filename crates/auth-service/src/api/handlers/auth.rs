use crate::api::dto::auth::{
    LoginRequest, LogoutRequest, LogoutResponse, RefreshTokenRequest, RegisterRequest,
};
use crate::api::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use common::AppError;
use validator::Validate;

/// Register a new user
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Delegate to AuthService
    let response = state.auth_service.register(request).await?;

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// Login with email and password
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Delegate to AuthService
    let response = state.auth_service.login(request).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Refresh with refresh token
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Response, AppError> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = state
        .auth_service
        .refresh_token(&request.refresh_token)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Logout user by revoking refresh token
///
/// Public endpoint following OAuth 2.0 Token Revocation standard (RFC 7009)
/// No access token required - validates refresh token instead
///
/// # Arguments
/// * `refresh_token` - The refresh token to revoke
///
/// # Returns
/// * `200 OK` - Token successfully revoked
/// * `400 Bad Request` - Invalid request format
/// * `401 Unauthorized` - Invalid or expired refresh token
pub async fn logout_handler(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Response, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Revoke token
    state
        .auth_service
        .logout(request.refresh_token.as_str())
        .await?;

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Successfully logged out".to_string(),
        }),
    )
        .into_response())
}
