use crate::domain::ConversationError;

#[derive(Debug, thiserror::Error)]
pub enum ConversationServiceError {
    #[error("Conversation not found")]
    NotFound,
    #[error("Not a member of this conversation")]
    NotMember,
    #[error("Already a member of this conversation")]
    AlreadyMember,
    #[error("Domain error: {0}")]
    DomainError(#[from] ConversationError),
    #[error("Repository error: {0}")]
    RepositoryError(String),
}
