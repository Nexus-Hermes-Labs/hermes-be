use crate::presentation::http::dto::{AuthResponse, AuthorizeUrlResponse, ClientInfo, GoogleCallbackRequest};
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
// OAUTH (GOOGLE) HANDLERS
// ============================================

/// GET /api/v1/auth/oauth/google
///
/// Begin Google sign-in. Returns the consent URL the SPA should navigate to.
/// A CSRF `state` + PKCE verifier are generated and stashed server-side.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/google",
    responses(
        (status = 200, description = "Authorization URL generated", body = AuthorizeUrlResponse),
        (status = 503, description = "Google OAuth is not configured")
    ),
    tag = "auth"
)]
pub async fn google_authorize_handler(
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let authorize_url = state
        .auth
        .oauth_service
        .start_google_authorization()
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(AuthorizeUrlResponse { authorize_url })).into_response())
}

/// POST /api/v1/auth/oauth/google/callback
///
/// Complete Google sign-in. The frontend brokers `{code, state}` here after
/// Google redirects to its callback route. Returns the same tokens as login.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/google/callback",
    request_body = GoogleCallbackRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid or expired state"),
        (status = 403, description = "Email not verified / account suspended or deleted"),
        (status = 502, description = "Google provider error"),
        (status = 503, description = "Google OAuth is not configured")
    ),
    tag = "auth"
)]
pub async fn google_callback_handler(
    State(state): State<AppState>,
    client_info: ClientInfo,
    Json(request): Json<GoogleCallbackRequest>,
) -> Result<Response, ApiError> {
    request.validate()?;

    let response = state
        .auth
        .oauth_service
        .complete_google_login(request.code, request.state, client_info)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(response)).into_response())
}
