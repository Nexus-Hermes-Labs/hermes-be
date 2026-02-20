use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserRelationshipError {
    #[error("Cannot have a relationship with oneself")]
    SelfRelationship,

    #[error("Message is too long (max 200 characters)")]
    MessageTooLong,

    #[error("Message is required for pending relationships")]
    MessageRequired,

    #[error("Message is only allowed for pending relationships")]
    MessageNotAllowed,

    #[error("Invalid state transition: Cannot accept a relationship that is not pending_incoming")]
    CannotAccept,

    #[error(
        "Invalid state transition: Cannot decline a relationship that is not pending_incoming"
    )]
    CannotDecline,

    #[error("Invalid state transition: The relationship is already blocked")]
    AlreadyBlocked,

    #[error("Invalid state transition: The relationship is not blocked")]
    NotBlocked,
}
