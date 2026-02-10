use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use crate::application::services::authentication::error::AuthApplicationError;

/// API error response
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

/// Error response JSON
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });

        (self.status, body).into_response()
    }
}

// ============================================
// CONVERSION FROM APPLICATION ERRORS
// ============================================

impl From<AuthAp> for ApiError {
    fn from(err: AuthApplicationError) -> Self {
        match err {
            // 400 Bad Request
            AuthApplicationError::EmailAlreadyExists => {
                ApiError::conflict("Email already registered")
            }
            AuthApplicationError::InvalidEmail => {
                ApiError::validation("Invalid email format")
            }
            AuthApplicationError::WeakPassword => {
                ApiError::validation("Password does not meet requirements")
            }
            AuthApplicationError::ValidationError(msg) => ApiError::validation(msg),

            // 401 Unauthorized
            AuthApplicationError::InvalidCredentials => {
                ApiError::unauthorized("Invalid email or password")
            }
            AuthApplicationError::InvalidRefreshToken => {
                ApiError::unauthorized("Invalid or expired refresh token")
            }
            AuthApplicationError::SessionNotFound => {
                ApiError::unauthorized("Session not found or expired")
            }

            // 403 Forbidden
            AuthApplicationError::AccountLocked { locked_until } => {
                let message = if let Some(until) = locked_until {
                    format!("Account locked until {}", until)
                } else {
                    "Account is locked".to_string()
                };
                ApiError::forbidden(message)
            }
            AuthApplicationError::AccountSuspended => {
                ApiError::forbidden("Account has been suspended")
            }
            AuthApplicationError::AccountDeleted => {
                ApiError::forbidden("Account has been deleted")
            }

            // 500 Internal Server Error
            _ => ApiError::internal("Internal server error"),
        }
    }
}

// ============================================
// CONVERSION FROM VALIDATION ERRORS
// ============================================

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::validation(err.to_string())
    }
}