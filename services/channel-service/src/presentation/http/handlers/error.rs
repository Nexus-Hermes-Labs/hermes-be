use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::application::ChannelServiceError;

/// JSON body returned to the client when an API error occurs.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g. `"not_found"`, `"validation_error"`).
    pub error: String,
    /// Human-readable description of what went wrong.
    pub message: String,
    /// Optional structured details (e.g. per-field validation errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Typed API error that implements [`axum::response::IntoResponse`].
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    error: String,
    message: String,
    details: Option<serde_json::Value>,
}

impl ApiError {
    /// Create an `ApiError` with an explicit HTTP status, error code, and message.
    pub fn new(status: StatusCode, error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    /// HTTP 400 – the request body or query parameters are invalid.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    /// HTTP 404 – the requested resource does not exist.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    /// HTTP 403 – the authenticated user lacks permission to perform the action.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }

    /// HTTP 422 – the request body is syntactically valid but fails domain validation.
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            message,
        )
    }

    /// HTTP 503 – a required upstream service (guild-service) is unavailable.
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            message,
        )
    }

    /// HTTP 500 – an unexpected server-side error occurred.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }

    /// Attach structured details to the error response.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
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

impl From<ChannelServiceError> for ApiError {
    fn from(err: ChannelServiceError) -> Self {
        match err {
            ChannelServiceError::ChannelNotFound => Self::not_found("Channel not found"),
            ChannelServiceError::GuildNotFound => Self::not_found("Guild not found"),
            ChannelServiceError::Forbidden(msg) => Self::forbidden(msg),
            ChannelServiceError::DomainError(e) => Self::bad_request(e.to_string()),
            ChannelServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
            ChannelServiceError::GrpcError(e) => {
                tracing::error!("Guild gRPC error: {}", e);
                Self::service_unavailable("Guild service is temporarily unavailable")
            }
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        Self::unprocessable("Validation failed").with_details(serde_json::json!({
            "validation_errors": errors.field_errors()
        }))
    }
}
