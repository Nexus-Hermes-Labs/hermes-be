use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::application::{ConversationServiceError, MessageServiceError, ReactionServiceError};
use crate::domain::{ConversationError, MessageError, ReactionError};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

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
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            message,
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }

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

// ── Error conversions ────────────────────────────────────────────────────────

impl From<MessageServiceError> for ApiError {
    fn from(err: MessageServiceError) -> Self {
        match err {
            MessageServiceError::NotFound => Self::not_found("Message not found"),
            MessageServiceError::Forbidden(msg) => Self::forbidden(msg),
            MessageServiceError::DomainError(e) => match e {
                MessageError::NotAuthor => {
                    Self::forbidden("Only the author can perform this action")
                }
                MessageError::MessageDeleted => {
                    Self::bad_request("Cannot modify a deleted message")
                }
                _ => Self::bad_request(e.to_string()),
            },
            MessageServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<ReactionServiceError> for ApiError {
    fn from(err: ReactionServiceError) -> Self {
        match err {
            ReactionServiceError::NotFound => Self::not_found("Reaction not found"),
            ReactionServiceError::MessageNotFound => Self::not_found("Message not found"),
            ReactionServiceError::AlreadyReacted => {
                Self::conflict("You have already reacted with this emoji")
            }
            ReactionServiceError::DomainError(e) => match e {
                ReactionError::InvalidEmoji => Self::bad_request("Invalid emoji"),
                ReactionError::AlreadyReacted => {
                    Self::conflict("You have already reacted with this emoji")
                }
                ReactionError::NotFound => Self::not_found("Reaction not found"),
            },
            ReactionServiceError::RepositoryError(e) => {
                tracing::error!("Repository error: {}", e);
                Self::internal("An error occurred while processing your request")
            }
        }
    }
}

impl From<ConversationServiceError> for ApiError {
    fn from(err: ConversationServiceError) -> Self {
        match err {
            ConversationServiceError::NotFound => Self::not_found("Conversation not found"),
            ConversationServiceError::NotMember => {
                Self::forbidden("You are not a member of this conversation")
            }
            ConversationServiceError::AlreadyMember => {
                Self::conflict("User is already a member of this conversation")
            }
            ConversationServiceError::DomainError(e) => match e {
                ConversationError::InvalidConversationType => {
                    Self::bad_request("Invalid conversation type")
                }
                ConversationError::InvalidDmMemberCount => {
                    Self::bad_request("A DM requires exactly 2 members")
                }
                ConversationError::InvalidGroupDmMemberCount => {
                    Self::bad_request("A group DM requires 3–10 members")
                }
                ConversationError::AlreadyMember => Self::conflict("User is already a member"),
                ConversationError::NotMember => Self::forbidden("User is not a member"),
                ConversationError::NotFound => Self::not_found("Conversation not found"),
            },
            ConversationServiceError::RepositoryError(e) => {
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
