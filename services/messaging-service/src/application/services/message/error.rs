use crate::domain::MessageError;

#[derive(Debug, thiserror::Error)]
pub enum MessageServiceError {
    #[error("Message not found")]
    NotFound,
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Domain error: {0}")]
    DomainError(#[from] MessageError),
    #[error("Repository error: {0}")]
    RepositoryError(String),
}
