use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("Invalid conversation type")]
    InvalidConversationType,
    #[error("A DM conversation requires exactly 2 members")]
    InvalidDmMemberCount,
    #[error("A group DM requires between 3 and 10 members")]
    InvalidGroupDmMemberCount,
    #[error("User is already a member of this conversation")]
    AlreadyMember,
    #[error("User is not a member of this conversation")]
    NotMember,
    #[error("Conversation not found")]
    NotFound,
}
