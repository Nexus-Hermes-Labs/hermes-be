use thiserror::Error;

/// Guild invite domain errors
#[derive(Debug, Error)]
pub enum GuildInviteError {
    #[error("Invite code not found")]
    NotFound,

    #[error("Invite has expired")]
    Expired,

    #[error("Invite has reached its maximum number of uses")]
    MaxUsesReached,

    #[error("Invite has already been revoked")]
    AlreadyRevoked,

    #[error("Only the creator or an admin can revoke this invite")]
    Unauthorized,
}
