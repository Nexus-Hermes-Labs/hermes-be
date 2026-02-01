use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserDomainError {
    // ================= User Profile =================
    #[error("Display name cannot be empty")]
    InvalidDisplayName,

    #[error("Display name is too long (max 100 characters)")]
    DisplayNameTooLong,

    #[error("Bio is too long (max 500 characters)")]
    BioTooLong,

    #[error("Invalid URL format (must start with http:// or https://)")]
    InvalidUrl,

    // ================= User Status =================
    #[error("Invalid user status value")]
    InvalidUserStatus,

    // ================= User Role =================
    #[error("Invalid user role value")]
    InvalidUserRole,

    // ================= Custom Status =================
    #[error("Custom status text cannot be empty")]
    CustomStatusTextEmpty,

    #[error("Custom status text is too long (max 128 characters)")]
    CustomStatusTextTooLong,

    #[error("Custom status emoji is too long (max 50 characters)")]
    CustomStatusEmojiTooLong,

    #[error("Custom status expiration must be in the future")]
    CustomStatusExpirationInPast,

    // ================= Privacy =================
    #[error("Invalid DM privacy setting")]
    InvalidDmPrivacy,

    #[error("Invalid friend request privacy setting")]
    InvalidFriendRequestPrivacy,

    #[error("Friend request not allowed by target user's privacy settings")]
    FriendRequestNotAllowed,

    // ================= User Relationship =================
    #[error("Cannot send friend request to yourself")]
    CannotBefriendSelf,

    #[error("Cannot block yourself")]
    CannotBlockSelf,

    #[error("Friend request message is too long (max 200 characters)")]
    MessageTooLong,

    #[error("Friend request message cannot be empty when provided")]
    EmptyMessage,

    #[error("Cannot accept a non-pending request")]
    CannotAcceptNonPendingRequest,

    #[error("Cannot decline a non-pending request")]
    CannotDeclineNonPendingRequest,

    #[error("Invalid relationship type value")]
    InvalidRelationshipType,
}