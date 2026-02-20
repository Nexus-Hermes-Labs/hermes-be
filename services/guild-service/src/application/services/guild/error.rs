use crate::domain::GuildError;
use common::infrastructure::persistence::error::RepositoryError;

/// Application errors for `GuildService`
#[derive(Debug, thiserror::Error)]
pub enum GuildServiceError {
    #[error("Guild not found")]
    GuildNotFound,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] GuildError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for GuildServiceError {
    fn from(e: RepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}
