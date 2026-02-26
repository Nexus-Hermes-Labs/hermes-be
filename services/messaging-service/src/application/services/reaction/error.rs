use crate::domain::ReactionError;

#[derive(Debug, thiserror::Error)]
pub enum ReactionServiceError {
    #[error("Reaction not found")]
    NotFound,
    #[error("Message not found")]
    MessageNotFound,
    #[error("Already reacted with this emoji")]
    AlreadyReacted,
    #[error("Domain error: {0}")]
    DomainError(#[from] ReactionError),
    #[error("Repository error: {0}")]
    RepositoryError(String),
}
