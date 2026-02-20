use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::application::{
    GuildInviteServiceError, GuildMemberServiceError, GuildRoleServiceError, GuildServiceError,
};

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
///
/// Construct via the convenience factories (`bad_request`, `not_found`, …)
/// rather than calling `new` directly.
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

    /// HTTP 409 – the request conflicts with existing state (e.g. duplicate resource).
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
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

    /// HTTP 500 – an unexpected server-side error occurred.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }

    /// Attach structured details to the error response (e.g. per-field validation errors).
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

impl From<GuildServiceError> for ApiError {
    fn from(err: GuildServiceError) -> Self {
        match err {
            GuildServiceError::GuildNotFound => Self::not_found("Guild not found"),
            GuildServiceError::Forbidden(msg) => Self::forbidden(msg),
            GuildServiceError::DomainError(e) => match e {
                crate::domain::GuildError::NotOwner => {
                    Self::forbidden("Only the guild owner can perform this action")
                }
                _ => Self::bad_request(e.to_string()),
            },
            GuildServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildMemberServiceError> for ApiError {
    fn from(err: GuildMemberServiceError) -> Self {
        match err {
            GuildMemberServiceError::GuildNotFound => Self::not_found("Guild not found"),
            GuildMemberServiceError::MemberNotFound => Self::not_found("Member not found"),
            GuildMemberServiceError::AlreadyMember => {
                Self::conflict("User is already a member of this guild")
            }
            GuildMemberServiceError::GuildFull => {
                Self::bad_request("Guild has reached its member limit")
            }
            GuildMemberServiceError::Forbidden(msg) => Self::forbidden(msg),
            GuildMemberServiceError::GuildDomainError(e) => Self::bad_request(e.to_string()),
            GuildMemberServiceError::MemberDomainError(e) => Self::bad_request(e.to_string()),
            GuildMemberServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildRoleServiceError> for ApiError {
    fn from(err: GuildRoleServiceError) -> Self {
        match err {
            GuildRoleServiceError::GuildNotFound => Self::not_found("Guild not found"),
            GuildRoleServiceError::RoleNotFound => Self::not_found("Role not found"),
            GuildRoleServiceError::RoleNameTaken => {
                Self::conflict("A role with this name already exists")
            }
            GuildRoleServiceError::CannotDeleteDefaultRole => {
                Self::bad_request("Cannot delete the @everyone role")
            }
            GuildRoleServiceError::Forbidden(msg) => Self::forbidden(msg),
            GuildRoleServiceError::DomainError(e) => Self::bad_request(e.to_string()),
            GuildRoleServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildInviteServiceError> for ApiError {
    fn from(err: GuildInviteServiceError) -> Self {
        match err {
            GuildInviteServiceError::GuildNotFound => Self::not_found("Guild not found"),
            GuildInviteServiceError::InviteNotFound => Self::not_found("Invite not found"),
            GuildInviteServiceError::InviteInvalid => {
                Self::bad_request("Invite is expired or exhausted")
            }
            GuildInviteServiceError::AlreadyMember => {
                Self::conflict("You are already a member of this guild")
            }
            GuildInviteServiceError::GuildFull => {
                Self::bad_request("Guild has reached its member limit")
            }
            GuildInviteServiceError::Forbidden(msg) => Self::forbidden(msg),
            GuildInviteServiceError::DomainError(e) => Self::bad_request(e.to_string()),
            GuildInviteServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
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
