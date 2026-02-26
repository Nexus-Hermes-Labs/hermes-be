use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReactionError {
    #[error("Emoji must be between 1 and 32 characters")]
    InvalidEmoji,
    #[error("Reaction already exists")]
    AlreadyReacted,
    #[error("Reaction not found")]
    NotFound,
}
