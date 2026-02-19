use thiserror::Error;

/// Guild member domain errors
#[derive(Debug, Error)]
pub enum GuildMemberError {
    #[error("Nickname must be between 1 and 32 characters")]
    InvalidNicknameLength,

    #[error("Nickname contains invalid characters")]
    InvalidNicknameFormat,

    #[error("Member not found")]
    NotFound,

    #[error("User is already a member of this guild")]
    AlreadyMember,

    #[error("User is banned from this guild")]
    Banned,

    #[error("Cannot remove the guild owner")]
    CannotRemoveOwner,

    #[error("Role is already assigned to this member")]
    RoleAlreadyAssigned,

    #[error("Role is not assigned to this member")]
    RoleNotAssigned,
}
