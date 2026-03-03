use thiserror::Error;

use crate::domain::ReactionError;

#[derive(Debug, Error)]
pub enum ReactionServiceError {
    #[error("Message not found")]
    MessageNotFound,

    #[error("Reaction not found")]
    ReactionNotFound,

    #[error("You have already reacted with this emoji")]
    AlreadyReacted,

    #[error("Domain error: {0}")]
    DomainError(#[from] ReactionError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
