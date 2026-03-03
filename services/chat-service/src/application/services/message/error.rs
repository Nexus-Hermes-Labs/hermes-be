use thiserror::Error;

use crate::domain::MessageError;

#[derive(Debug, Error)]
pub enum MessageServiceError {
    #[error("Message not found")]
    NotFound,

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] MessageError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
