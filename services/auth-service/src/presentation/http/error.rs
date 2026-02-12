// presentation/api/errors/api_error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use validator::ValidationErrors;
use crate::application::services::authentication::error::AuthApplicationError;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    pub code: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
            code: self.code,
        });
        (self.status, body).into_response()
    }
}

impl From<AuthApplicationError> for ApiError {
    fn from(err: AuthApplicationError) -> Self {
        match err {
            // 400 Bad Request
            AuthApplicationError::InvalidEmail(email) => ApiError {
                status: StatusCode::BAD_REQUEST,
                message: format!("Invalid email format: {}", email),
                code: "INVALID_EMAIL".to_string(),
            },
            AuthApplicationError::WeakPassword => ApiError {
                status: StatusCode::BAD_REQUEST,
                message: "Password must be at least 8 characters".to_string(),
                code: "WEAK_PASSWORD".to_string(),
            },

            // 401 Unauthorized
            AuthApplicationError::InvalidCredentials => ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid email or password".to_string(),
                code: "INVALID_CREDENTIALS".to_string(),
            },

            // 403 Forbidden
            AuthApplicationError::AccountLocked { locked_until } => ApiError {
                status: StatusCode::FORBIDDEN,
                message: format!("Account locked until {}", locked_until),
                code: "ACCOUNT_LOCKED".to_string(),
            },

            // 409 Conflict
            AuthApplicationError::EmailAlreadyExists(_) => ApiError {
                status: StatusCode::CONFLICT,
                message: "This email is already registered".to_string(),
                code: "EMAIL_ALREADY_EXISTS".to_string(),
            },

            // 500 Internal Server Error
            AuthApplicationError::HashingFailed
            | AuthApplicationError::TokenGenerationFailed(_)
            | AuthApplicationError::Internal(_) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "An error occurred. Please try again later.".to_string(),
                code: "INTERNAL_ERROR".to_string(),
            },

            _ => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Internal server error".to_string(),
                code: "UNKNOWN_ERROR".to_string(),
            },
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        // Validation hatalarını user_profile-friendly formata çevir
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Invalid value".to_string()))
                })
            })
            .collect();

        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "VALIDATION_ERROR".to_string(),
            message: error_messages.join(", "),
        }
    }
}