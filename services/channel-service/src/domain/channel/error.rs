use thiserror::Error;

/// Channel domain errors
#[derive(Debug, Error)]
pub enum ChannelError {
    // ============================================
    // NAME ERRORS
    // ============================================
    #[error("Channel name must be between 1 and 100 characters")]
    InvalidNameLength,

    #[error("Channel name contains invalid characters")]
    InvalidNameFormat,

    // ============================================
    // DESCRIPTION ERRORS
    // ============================================
    #[error("Channel description is too long (max 1024 characters)")]
    DescriptionTooLong,

    // ============================================
    // STRUCTURAL ERRORS
    // ============================================
    #[error("Parent channel ID cannot be the same as the channel ID")]
    InvalidParentId,

    #[error("Invalid channel type: {0}")]
    InvalidChannelType(String),

    // ============================================
    // GENERAL ERRORS
    // ============================================
    #[error("Channel not found")]
    NotFound,

    #[error("Channel has been deleted")]
    ChannelDeleted,
}
