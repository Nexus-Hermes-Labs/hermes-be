use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::error;

/// Application-wide error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Domain Errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {entity_type}")]
    NotFound { entity_type: String },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Message queue error: {0}")]
    MessageQueue(String),

    #[error("JWT error: {0}")]
    Jwt(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Account locked: {0}")]
    Locked(String)
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Locked(_) => StatusCode::LOCKED,
            Self::Database(_)
            | Self::Cache(_)
            | Self::InternalServerError(_)
            | AppError::MessageQueue(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Conflict(_) => "CONFLICT",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Cache(_) => "CACHE_ERROR",
            Self::Locked(_) => "ACCOUNT_LOCKED_ERROR",
            Self::InternalServerError(_) => "INTERNAL_ERROR",
            Self::Jwt(_) => "JWT_ERROR",
            Self::Config(_) => "CONFIG_ERROR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::MessageQueue(_) => "MESSAGE_QUEUE_ERROR",
        }
    }

    pub fn should_log(&self) -> bool {
        matches!(
            self,
            Self::Database(_) | Self::Cache(_) | Self::InternalServerError(_) | Self::Config(_)
        )
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log internal errors
        if self.should_log() {
            error!(
                error = ?self,
                error_code = self.error_code(),
                "Application error occurred"
            );
        }

        let status = self.status_code();
        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details: None,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

// Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
