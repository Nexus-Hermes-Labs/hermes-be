use common::infrastructure::persistence::error::RepositoryError;
use crate::domain::GuildRoleError;

/// Application errors for GuildRoleService
#[derive(Debug, thiserror::Error)]
pub enum GuildRoleServiceError {
    #[error("Guild not found")]
    GuildNotFound,

    #[error("Role not found")]
    RoleNotFound,

    #[error("A role with this name already exists")]
    RoleNameTaken,

    #[error("Cannot delete the @everyone role")]
    CannotDeleteDefaultRole,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Domain error: {0}")]
    DomainError(#[from] GuildRoleError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for GuildRoleServiceError {
    fn from(e: RepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}
