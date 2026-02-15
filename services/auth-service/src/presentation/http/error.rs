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

            // 401 Unauthorized
            AuthApplicationError::InvalidToken => ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid or expired token".to_string(),
                code: "INVALID_TOKEN".to_string(),
            },

            // 403 Forbidden
            AuthApplicationError::AccountSuspended => ApiError {
                status: StatusCode::FORBIDDEN,
                message: "Account is suspended".to_string(),
                code: "ACCOUNT_SUSPENDED".to_string(),
            },
            AuthApplicationError::AccountDeleted => ApiError {
                status: StatusCode::FORBIDDEN,
                message: "Account is deleted".to_string(),
                code: "ACCOUNT_DELETED".to_string(),
            },
            AuthApplicationError::AccountDeactivated => ApiError {
                status: StatusCode::FORBIDDEN,
                message: "Account is deactivated".to_string(),
                code: "ACCOUNT_DEACTIVATED".to_string(),
            },
            AuthApplicationError::EmailNotVerified => ApiError {
                status: StatusCode::FORBIDDEN,
                message: "Email not verified".to_string(),
                code: "EMAIL_NOT_VERIFIED".to_string(),
            },

            // 404 Not Found
            AuthApplicationError::UserNotFound(_)
            | AuthApplicationError::UserNotFoundByEmail(_) => ApiError {
                status: StatusCode::NOT_FOUND,
                message: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            },

            // 409 Conflict
            AuthApplicationError::UsernameAlreadyExists(_) => ApiError {
                status: StatusCode::CONFLICT,
                message: "This username is already taken".to_string(),
                code: "USERNAME_ALREADY_EXISTS".to_string(),
            },

            // 502 Bad Gateway (upstream service failure)
            AuthApplicationError::UserProfileCreationFailed(ref msg) => {
                tracing::error!("User profile creation failed: {}", msg);
                ApiError {
                    status: StatusCode::BAD_GATEWAY,
                    message: "Failed to create user profile. Please try again later.".to_string(),
                    code: "USER_SERVICE_ERROR".to_string(),
                }
            },
            AuthApplicationError::UserServiceError(ref msg) => {
                tracing::error!("User service communication error: {}", msg);
                ApiError {
                    status: StatusCode::BAD_GATEWAY,
                    message: "User service is unavailable. Please try again later.".to_string(),
                    code: "USER_SERVICE_UNAVAILABLE".to_string(),
                }
            },

            // 500 Internal Server Error
            AuthApplicationError::HashingFailed
            | AuthApplicationError::TokenGenerationFailed(_)
            | AuthApplicationError::Internal(_)
            | AuthApplicationError::RepositoryError(_) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "An error occurred. Please try again later.".to_string(),
                code: "INTERNAL_ERROR".to_string(),
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