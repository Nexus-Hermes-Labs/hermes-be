use common::infrastructure::persistence::error::RepositoryError;
use crate::domain::{GuildError, GuildMemberError};

/// Application errors for GuildMemberService
#[derive(Debug, thiserror::Error)]
pub enum GuildMemberServiceError {
    #[error("Guild not found")]
    GuildNotFound,

    #[error("Member not found")]
    MemberNotFound,

    #[error("User is already a member of this guild")]
    AlreadyMember,

    #[error("Guild is full")]
    GuildFull,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    GuildDomainError(#[from] GuildError),

    #[error("Member domain error: {0}")]
    MemberDomainError(#[from] GuildMemberError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for GuildMemberServiceError {
    fn from(e: RepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}
