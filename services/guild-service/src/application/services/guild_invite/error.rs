use common::infrastructure::persistence::error::RepositoryError;
use crate::domain::GuildInviteError;

/// Application errors for GuildInviteService
#[derive(Debug, thiserror::Error)]
pub enum GuildInviteServiceError {
    #[error("Guild not found")]
    GuildNotFound,

    #[error("Invite not found")]
    InviteNotFound,

    #[error("Invite is no longer valid")]
    InviteInvalid,

    #[error("User is already a member of this guild")]
    AlreadyMember,

    #[error("Guild is full")]
    GuildFull,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] GuildInviteError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for GuildInviteServiceError {
    fn from(e: RepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}
