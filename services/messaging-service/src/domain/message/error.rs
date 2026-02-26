use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Message content must be between 1 and 2000 characters")]
    InvalidContentLength,
    #[error("Message content contains invalid characters (null bytes not allowed)")]
    InvalidContentFormat,
    #[error("Invalid message type")]
    InvalidMessageType,
    #[error("Only the message author can perform this action")]
    NotAuthor,
    #[error("Message has already been deleted")]
    MessageDeleted,
    #[error("Message not found")]
    NotFound,
}
