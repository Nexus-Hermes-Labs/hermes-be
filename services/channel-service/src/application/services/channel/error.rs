use crate::domain::ChannelError;
use common::infrastructure::persistence::error::RepositoryError;

/// Application errors for `ChannelService`
#[derive(Debug, thiserror::Error)]
pub enum ChannelServiceError {
    #[error("Channel not found")]
    ChannelNotFound,

    #[error("Guild not found")]
    GuildNotFound,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] ChannelError),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Guild service unavailable: {0}")]
    GrpcError(String),
}

impl From<RepositoryError> for ChannelServiceError {
    fn from(e: RepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}
