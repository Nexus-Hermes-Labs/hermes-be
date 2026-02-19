use thiserror::Error;

/// Guild role domain errors
#[derive(Debug, Error)]
pub enum GuildRoleError {
    #[error("Role name must be between 1 and 100 characters")]
    InvalidNameLength,

    #[error("Cannot delete the @everyone role")]
    CannotDeleteDefaultRole,

    #[error("Cannot edit a role with higher position than your own")]
    InsufficientRolePosition,

    #[error("Role not found")]
    NotFound,

    #[error("A role with this name already exists in the guild")]
    NameAlreadyExists,

    #[error("Invalid color format (must be a valid RGB hex, e.g. #FFFFFF)")]
    InvalidColor,
}
