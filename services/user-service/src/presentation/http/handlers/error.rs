use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::application::services::{UserPrivacyServiceError, UserProfileServiceError};

/// API Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// API Error
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    error: String,
    message: String,
    details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(status: StatusCode, error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    // ============================================
    // CONSTRUCTORS
    // ============================================

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, "too_many_requests", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, "validation_error", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.error,
            message: self.message,
            details: self.details,
        });

        (self.status, body).into_response()
    }
}

// ============================================
// ERROR CONVERSIONS
// ============================================

impl From<UserProfileServiceError> for ApiError {
    fn from(err: UserProfileServiceError) -> Self {
        match err {
            UserProfileServiceError::ProfileNotFound => {
                ApiError::not_found("User profile not found")
            }
            UserProfileServiceError::UsernameAlreadyTaken => {
                ApiError::conflict("Username is already taken")
            }
            UserProfileServiceError::InvalidUsername(msg) => ApiError::validation(msg),
            UserProfileServiceError::DomainError(e) => ApiError::bad_request(e.to_string()),
            UserProfileServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<UserPrivacyServiceError> for ApiError {
    fn from(err: UserPrivacyServiceError) -> Self {
        match err {
            UserPrivacyServiceError::PrivacySettingsNotFound { .. } => {
                ApiError::not_found("Privacy settings not found")
            }
            UserPrivacyServiceError::DmNotAllowed => {
                ApiError::forbidden("User does not accept DMs from this relationship type")
            }
            UserPrivacyServiceError::FriendRequestNotAllowed => ApiError::forbidden(
                "User does not accept friend requests from this relationship type",
            ),
            UserPrivacyServiceError::DomainError(e) => ApiError::bad_request(e.to_string()),
            UserPrivacyServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        ApiError::validation("Validation failed").with_details(serde_json::json!({
            "validation_errors": errors.field_errors()
        }))
    }
}
