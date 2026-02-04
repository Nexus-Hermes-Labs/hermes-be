use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UserRelationshipDomainError {
    // =====================================================
    // VALIDATION ERRORS
    // =====================================================
    #[error("Cannot send friend request to yourself")]
    CannotBefriendSelf,

    #[error("Cannot block yourself")]
    CannotBlockSelf,

    #[error("Message is too long (max 200 characters)")]
    MessageTooLong,

    #[error("Message cannot be empty when provided")]
    MessageEmpty,

    #[error("Invalid relationship type: {0}")]
    InvalidRelationshipType(String),

    // =====================================================
    // STATE TRANSITION ERRORS
    // =====================================================
    #[error("Cannot accept a non-pending friend request")]
    CannotAcceptNonPendingRequest,

    #[error("Cannot decline a non-pending friend request")]
    CannotDeclineNonPendingRequest,

    #[error("Friend request has already been processed")]
    RequestAlreadyProcessed,

    #[error("Cannot modify a friendship that doesn't exist")]
    FriendshipDoesNotExist,

    #[error("Relationship already exists between these users")]
    RelationshipAlreadyExists,

    #[error("Cannot send friend request: users are already friends")]
    AlreadyFriends,

    #[error("Cannot send friend request: you have already sent a pending request")]
    PendingRequestAlreadySent,

    #[error("Cannot send friend request: target user has already sent you a request")]
    PendingRequestAlreadyReceived,

    #[error("User is blocked and cannot interact")]
    UserIsBlocked,

    #[error("You have blocked this user and cannot interact")]
    YouHaveBlockedUser,

    // =====================================================
    // PRIVACY ERRORS
    // =====================================================
    #[error("Cannot send friend request: target user's privacy settings do not allow it")]
    FriendRequestNotAllowedByPrivacy,

    #[error("Cannot send DM: target user's privacy settings do not allow it")]
    DmNotAllowedByPrivacy,

    #[error("User's online status is hidden due to privacy settings")]
    OnlineStatusHidden,

    // =====================================================
    // AUTHORIZATION ERRORS
    // =====================================================
    #[error("Not authorized to modify this relationship")]
    NotAuthorized,

    #[error("Only the receiver can accept this friend request")]
    OnlyReceiverCanAccept,

    #[error("Only the receiver can decline this friend request")]
    OnlyReceiverCanDecline,

    #[error("Both users can remove a friendship")]
    CannotRemoveFriendship,

    // =====================================================
    // BUSINESS RULE VIOLATIONS
    // =====================================================
    #[error("Friend limit reached (max {0} friends)")]
    FriendLimitReached(usize),

    #[error("Pending friend request limit reached (max {0} pending)")]
    PendingRequestLimitReached(usize),

    #[error("Block list limit reached (max {0} blocked users)")]
    BlockLimitReached(usize),

    #[error("Cannot send friend requests: user account is restricted")]
    AccountRestricted,

    #[error("Cannot interact: user account is deleted or deactivated")]
    UserAccountInactive,

    // =====================================================
    // AGGREGATE CONSISTENCY ERRORS
    // =====================================================
    #[error("Bidirectional relationship is out of sync")]
    BidirectionalSyncError,

    #[error("Invalid relationship state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
}

// =====================================================
// HELPER IMPLEMENTATIONS
// =====================================================
impl UserRelationshipDomainError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CannotBefriendSelf => "cannot_befriend_self",
            Self::CannotBlockSelf => "cannot_block_self",
            Self::MessageTooLong => "message_too_long",
            Self::MessageEmpty => "message_empty",
            Self::InvalidRelationshipType(_) => "invalid_relationship_type",

            Self::CannotAcceptNonPendingRequest => "cannot_accept_non_pending_request",
            Self::CannotDeclineNonPendingRequest => "cannot_decline_non_pending_request",
            Self::RequestAlreadyProcessed => "request_already_processed",
            Self::FriendshipDoesNotExist => "friendship_does_not_exist",
            Self::RelationshipAlreadyExists => "relationship_already_exists",
            Self::AlreadyFriends => "already_friends",
            Self::PendingRequestAlreadySent => "pending_request_already_sent",
            Self::PendingRequestAlreadyReceived => "pending_request_already_received",
            Self::UserIsBlocked => "user_is_blocked",
            Self::YouHaveBlockedUser => "you_have_blocked_user",

            Self::FriendRequestNotAllowedByPrivacy => "friend_request_not_allowed_by_privacy",
            Self::DmNotAllowedByPrivacy => "dm_not_allowed_by_privacy",
            Self::OnlineStatusHidden => "online_status_hidden",

            Self::NotAuthorized => "not_authorized",
            Self::OnlyReceiverCanAccept => "only_receiver_can_accept",
            Self::OnlyReceiverCanDecline => "only_receiver_can_decline",
            Self::CannotRemoveFriendship => "cannot_remove_friendship",

            Self::FriendLimitReached(_) => "friend_limit_reached",
            Self::PendingRequestLimitReached(_) => "pending_request_limit_reached",
            Self::BlockLimitReached(_) => "block_limit_reached",
            Self::AccountRestricted => "account_restricted",
            Self::UserAccountInactive => "user_account_inactive",

            Self::BidirectionalSyncError => "bidirectional_sync_error",
            Self::InvalidStateTransition { .. } => "invalid_state_transition",
        }
    }
}

impl TryFrom<&str> for UserRelationshipDomainError {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "cannot_befriend_self" => Self::CannotBefriendSelf,
            "cannot_block_self" => Self::CannotBlockSelf,
            "message_too_long" => Self::MessageTooLong,
            "message_empty" => Self::MessageEmpty,
            "invalid_relationship_type" => Self::InvalidRelationshipType("unknown".into()),

            "cannot_accept_non_pending_request" => Self::CannotAcceptNonPendingRequest,
            "cannot_decline_non_pending_request" => Self::CannotDeclineNonPendingRequest,
            "request_already_processed" => Self::RequestAlreadyProcessed,
            "friendship_does_not_exist" => Self::FriendshipDoesNotExist,
            "relationship_already_exists" => Self::RelationshipAlreadyExists,
            "already_friends" => Self::AlreadyFriends,
            "pending_request_already_sent" => Self::PendingRequestAlreadySent,
            "pending_request_already_received" => Self::PendingRequestAlreadyReceived,
            "user_is_blocked" => Self::UserIsBlocked,
            "you_have_blocked_user" => Self::YouHaveBlockedUser,

            "friend_request_not_allowed_by_privacy" => Self::FriendRequestNotAllowedByPrivacy,
            "dm_not_allowed_by_privacy" => Self::DmNotAllowedByPrivacy,
            "online_status_hidden" => Self::OnlineStatusHidden,

            "not_authorized" => Self::NotAuthorized,
            "only_receiver_can_accept" => Self::OnlyReceiverCanAccept,
            "only_receiver_can_decline" => Self::OnlyReceiverCanDecline,
            "cannot_remove_friendship" => Self::CannotRemoveFriendship,

            "friend_limit_reached" => Self::FriendLimitReached(0),
            "pending_request_limit_reached" => Self::PendingRequestLimitReached(0),
            "block_limit_reached" => Self::BlockLimitReached(0),
            "account_restricted" => Self::AccountRestricted,
            "user_account_inactive" => Self::UserAccountInactive,

            "bidirectional_sync_error" => Self::BidirectionalSyncError,
            "invalid_state_transition" => Self::InvalidStateTransition {
                from: "unknown".into(),
                to: "unknown".into(),
            },

            _ => return Err(()),
        })
    }
}


impl UserRelationshipDomainError {
    /// Check if error is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            Self::CannotBefriendSelf
                | Self::CannotBlockSelf
                | Self::MessageTooLong
                | Self::MessageEmpty
                | Self::InvalidRelationshipType(_)
        )
    }

    /// Check if error is a state error
    pub fn is_state_error(&self) -> bool {
        matches!(
            self,
            Self::CannotAcceptNonPendingRequest
                | Self::CannotDeclineNonPendingRequest
                | Self::RequestAlreadyProcessed
                | Self::AlreadyFriends
                | Self::InvalidStateTransition { .. }
        )
    }

    /// Check if error is a privacy error
    pub fn is_privacy_error(&self) -> bool {
        matches!(
            self,
            Self::FriendRequestNotAllowedByPrivacy
                | Self::DmNotAllowedByPrivacy
                | Self::OnlineStatusHidden
        )
    }

    /// Check if error is an authorization error
    pub fn is_authorization_error(&self) -> bool {
        matches!(
            self,
            Self::NotAuthorized | Self::OnlyReceiverCanAccept | Self::OnlyReceiverCanDecline
        )
    }

    /// Check if error is a business rule violation
    pub fn is_business_rule_violation(&self) -> bool {
        matches!(
            self,
            Self::FriendLimitReached(_)
                | Self::PendingRequestLimitReached(_)
                | Self::BlockLimitReached(_)
                | Self::AccountRestricted
                | Self::UserAccountInactive
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let validation_err = UserRelationshipDomainError::MessageTooLong;
        assert!(validation_err.is_validation_error());
        assert!(!validation_err.is_privacy_error());

        let privacy_err = UserRelationshipDomainError::FriendRequestNotAllowedByPrivacy;
        assert!(privacy_err.is_privacy_error());
        assert!(!privacy_err.is_validation_error());

        let state_err = UserRelationshipDomainError::AlreadyFriends;
        assert!(state_err.is_state_error());
        assert!(!state_err.is_authorization_error());
    }

    #[test]
    fn test_error_messages() {
        let err = UserRelationshipDomainError::FriendLimitReached(100);
        assert_eq!(err.to_string(), "Friend limit reached (max 100 friends)");

        let err = UserRelationshipDomainError::InvalidStateTransition {
            from: "pending_outgoing".to_string(),
            to: "blocked".to_string(),
        };
        assert!(err.to_string().contains("pending_outgoing"));
        assert!(err.to_string().contains("blocked"));
    }
}
