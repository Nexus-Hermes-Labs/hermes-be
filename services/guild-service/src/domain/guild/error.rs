use thiserror::Error;

/// Guild domain errors
#[derive(Debug, Error)]
pub enum GuildError {
    // ============================================
    // NAME ERRORS
    // ============================================
    #[error("Guild name must be between 2 and 100 characters")]
    InvalidNameLength,

    #[error("Guild name contains invalid characters")]
    InvalidNameFormat,

    // ============================================
    // DESCRIPTION ERRORS
    // ============================================
    #[error("Guild description is too long (max 1000 characters)")]
    DescriptionTooLong,

    // ============================================
    // URL ERRORS
    // ============================================
    #[error("Icon URL is invalid (must start with http:// or https://)")]
    InvalidIconUrl,

    #[error("Banner URL is invalid (must start with http:// or https://)")]
    InvalidBannerUrl,

    // ============================================
    // OWNERSHIP ERRORS
    // ============================================
    #[error("Only the guild owner can perform this action")]
    NotOwner,

    #[error("Cannot transfer ownership to a non-member")]
    TransferOwnershipToNonMember,

    // ============================================
    // MEMBER LIMIT ERRORS
    // ============================================
    #[error("Guild has reached the maximum number of members")]
    MemberLimitReached,

    // ============================================
    // GENERAL ERRORS
    // ============================================
    #[error("Guild not found")]
    NotFound,

    #[error("Guild has been deleted")]
    GuildDeleted,
}
