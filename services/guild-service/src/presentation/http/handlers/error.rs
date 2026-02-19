use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::application::{
    GuildInviteServiceError, GuildMemberServiceError, GuildRoleServiceError, GuildServiceError,
};

/// API Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Typed API error
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    error: String,
    message: String,
    details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(
        status: StatusCode,
        error: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status,
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

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

    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, "validation_error", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
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
            GuildServiceError::GuildNotFound => ApiError::not_found("Guild not found"),
            GuildServiceError::Forbidden(msg) => ApiError::forbidden(msg),
            GuildServiceError::DomainError(e) => ApiError::bad_request(e.to_string()),
            GuildServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildMemberServiceError> for ApiError {
    fn from(err: GuildMemberServiceError) -> Self {
        match err {
            GuildMemberServiceError::GuildNotFound => ApiError::not_found("Guild not found"),
            GuildMemberServiceError::MemberNotFound => ApiError::not_found("Member not found"),
            GuildMemberServiceError::AlreadyMember => {
                ApiError::conflict("User is already a member of this guild")
            }
            GuildMemberServiceError::GuildFull => {
                ApiError::bad_request("Guild has reached its member limit")
            }
            GuildMemberServiceError::Forbidden(msg) => ApiError::forbidden(msg),
            GuildMemberServiceError::GuildDomainError(e) => ApiError::bad_request(e.to_string()),
            GuildMemberServiceError::MemberDomainError(e) => ApiError::bad_request(e.to_string()),
            GuildMemberServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildRoleServiceError> for ApiError {
    fn from(err: GuildRoleServiceError) -> Self {
        match err {
            GuildRoleServiceError::GuildNotFound => ApiError::not_found("Guild not found"),
            GuildRoleServiceError::RoleNotFound => ApiError::not_found("Role not found"),
            GuildRoleServiceError::RoleNameTaken => {
                ApiError::conflict("A role with this name already exists")
            }
            GuildRoleServiceError::CannotDeleteDefaultRole => {
                ApiError::bad_request("Cannot delete the @everyone role")
            }
            GuildRoleServiceError::Forbidden(msg) => ApiError::forbidden(msg),
            GuildRoleServiceError::DomainError(e) => ApiError::bad_request(e.to_string()),
            GuildRoleServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<GuildInviteServiceError> for ApiError {
    fn from(err: GuildInviteServiceError) -> Self {
        match err {
            GuildInviteServiceError::GuildNotFound => ApiError::not_found("Guild not found"),
            GuildInviteServiceError::InviteNotFound => ApiError::not_found("Invite not found"),
            GuildInviteServiceError::InviteInvalid => {
                ApiError::bad_request("Invite is expired or exhausted")
            }
            GuildInviteServiceError::AlreadyMember => {
                ApiError::conflict("You are already a member of this guild")
            }
            GuildInviteServiceError::GuildFull => {
                ApiError::bad_request("Guild has reached its member limit")
            }
            GuildInviteServiceError::Forbidden(msg) => ApiError::forbidden(msg),
            GuildInviteServiceError::DomainError(e) => ApiError::bad_request(e.to_string()),
            GuildInviteServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                ApiError::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        ApiError::unprocessable("Validation failed").with_details(serde_json::json!({
            "validation_errors": errors.field_errors()
        }))
    }
}

impl ApiError {
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
