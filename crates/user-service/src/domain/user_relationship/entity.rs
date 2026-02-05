use crate::domain::user::User;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user_relationship::valueobject::RelationshipType;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// UserRelationship aggregate root
/// Represents a directed edge in the user relationship graph
#[derive(Debug, Clone, PartialEq)]
pub struct UserRelationship {
    id: Uuid,
    user_id: Uuid,        // Owner of this edge (perspective)
    target_user_id: Uuid, // Target user
    relationship_type: RelationshipType,
    message: Option<String>, // Only for pending requests
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// =====================================================
// CONSTRUCTORS & GETTERS
// =====================================================

impl UserRelationship {
    /// Private constructor - use factory methods instead
    fn new(
        user_id: Uuid,
        target_user_id: Uuid,
        relationship_type: RelationshipType,
        message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            target_user_id,
            relationship_type,
            message,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Reconstruct from persistence (used by repository)
    pub fn reconstruct(
        id: Uuid,
        user_id: Uuid,
        target_user_id: Uuid,
        relationship_type: RelationshipType,
        message: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            target_user_id,
            relationship_type,
            message,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn target_user_id(&self) -> &Uuid {
        &self.target_user_id
    }

    pub fn relationship_type(&self) -> &RelationshipType {
        &self.relationship_type
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

// =====================================================
// FACTORY METHODS
// =====================================================

impl UserRelationship {
    /// Create a friend request (creates pending_outgoing)
    /// The database trigger will automatically create the reverse (pending_incoming)
    ///
    /// # Business Rules
    /// - Cannot befriend yourself
    /// - Message must be ≤ 200 characters
    /// - Message cannot be empty string if provided
    pub fn create_friend_request(
        sender_id: &Uuid,
        receiver_id: &Uuid,
        message: Option<String>,
    ) -> Result<Self, UserRelationshipDomainError> {
        // Rule: Cannot befriend yourself
        if sender_id == receiver_id {
            return Err(UserRelationshipDomainError::CannotBefriendSelf);
        }

        // Rule: Validate message
        if let Some(msg) = &message {
            if msg.trim().is_empty() {
                return Err(UserRelationshipDomainError::MessageEmpty);
            }
            if msg.len() > 200 {
                return Err(UserRelationshipDomainError::MessageTooLong);
            }
        }

        Ok(Self::new(
            *sender_id,
            *receiver_id,
            RelationshipType::PendingOutgoing,
            message,
        ))
    }

    /// Validate friend request can be sent
    pub fn validate_friend_request(
        sender_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<(), UserRelationshipDomainError> {
        // Already checked in entity, but double-check
        if sender_id == receiver_id {
            return Err(UserRelationshipDomainError::CannotBefriendSelf);
        }

        // Future validations:
        // - Check if sender is blocked by receiver
        // - Check max friends limit
        // - Check if receiver accepts friend requests

        Ok(())
    }


    /// Create a block relationship (unidirectional)
    ///
    /// # Business Rules
    /// - Cannot block yourself
    /// - Blocks are unidirectional (no reverse relationship created)
    pub fn create_block(
        blocker_id: Uuid,
        blocked_id: Uuid,
    ) -> Result<Self, UserRelationshipDomainError> {
        // Rule: Cannot block yourself
        Self::validate_block(&blocker_id, &blocked_id)?;

        Ok(Self::new(
            blocker_id,
            blocked_id,
            RelationshipType::Blocked,
            None, // Blocks never have messages
        ))
    }

    /// Validate block can be created
    pub fn validate_block(
        blocker_id: &Uuid,
        blocked_id: &Uuid,
    ) -> Result<(), UserRelationshipDomainError> {
        if blocker_id == blocked_id {
            return Err(UserRelationshipDomainError::CannotBlockSelf);
        }

        Ok(())
    }
}

// =====================================================
// DOMAIN BEHAVIORS (State Transitions)
// =====================================================

impl UserRelationship {
    /// Accept a friend request
    /// Converts: pending_incoming → friend
    ///
    /// # Business Rules
    /// - Can only accept pending_incoming requests
    /// - Message is cleared when accepting
    /// - Trigger updates the reverse relationship to 'friend'
    pub fn accept(&mut self) -> Result<(), UserRelationshipDomainError> {
        match self.relationship_type {
            RelationshipType::PendingIncoming => {
                self.relationship_type = RelationshipType::Friend;
                self.message = None; // Clear message when accepting
                self.updated_at = Utc::now();
                Ok(())
            }
            RelationshipType::Friend => Err(UserRelationshipDomainError::AlreadyFriends),
            RelationshipType::PendingOutgoing => {
                Err(UserRelationshipDomainError::CannotAcceptNonPendingRequest)
            }
            RelationshipType::Blocked => Err(UserRelationshipDomainError::UserIsBlocked),
        }
    }

    /// Decline a friend request
    /// Note: The relationship should be deleted after this
    ///
    /// # Business Rules
    /// - Can only decline pending_incoming requests
    pub fn decline(&mut self) -> Result<(), UserRelationshipDomainError> {
        match self.relationship_type {
            RelationshipType::PendingIncoming => {
                // The application service should delete this relationship
                self.updated_at = Utc::now();
                Ok(())
            }
            RelationshipType::Friend => Err(UserRelationshipDomainError::AlreadyFriends),
            RelationshipType::PendingOutgoing => {
                Err(UserRelationshipDomainError::CannotDeclineNonPendingRequest)
            }
            RelationshipType::Blocked => Err(UserRelationshipDomainError::UserIsBlocked),
        }
    }

    /// Cancel a sent friend request
    /// Note: The relationship should be deleted after this
    ///
    /// # Business Rules
    /// - Can only cancel pending_outgoing requests
    pub fn cancel(&mut self) -> Result<(), UserRelationshipDomainError> {
        match self.relationship_type {
            RelationshipType::PendingOutgoing => {
                self.updated_at = Utc::now();
                Ok(())
            }
            RelationshipType::Friend => Err(UserRelationshipDomainError::AlreadyFriends),
            RelationshipType::PendingIncoming => Err(UserRelationshipDomainError::NotAuthorized),
            RelationshipType::Blocked => Err(UserRelationshipDomainError::UserIsBlocked),
        }
    }
}

// =====================================================
// DOMAIN QUERIES
// =====================================================

impl UserRelationship {
    /// Check if this is a friend relationship
    pub fn is_friend(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::Friend)
    }

    /// Check if this is a pending incoming request
    pub fn is_pending_incoming(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::PendingIncoming)
    }

    /// Check if this is a pending outgoing request
    pub fn is_pending_outgoing(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::PendingOutgoing)
    }

    /// Check if this is any kind of pending request
    pub fn is_pending(&self) -> bool {
        self.is_pending_incoming() || self.is_pending_outgoing()
    }

    /// Check if this is a block
    pub fn is_blocked(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::Blocked)
    }

    /// Check if the relationship can be modified by the given user
    pub fn can_be_modified_by(&self, user_id: &Uuid) -> bool {
        &self.user_id == user_id
    }

    /// Check if the user is the receiver of this relationship
    pub fn is_receiver(&self, user_id: &Uuid) -> bool {
        &self.target_user_id == user_id
    }

    /// Check if the user is the sender of this relationship
    pub fn is_sender(&self, user_id: &Uuid) -> bool {
        &self.user_id == user_id
    }

    /// Get the other user in the relationship
    pub fn get_other_user_id(&self, user_id: &Uuid) -> Option<Uuid> {
        if &self.user_id == user_id {
            Some(self.target_user_id)
        } else if &self.target_user_id == user_id {
            Some(self.user_id)
        } else {
            None
        }
    }
}

// =====================================================
// VALIDATION
// =====================================================

impl UserRelationship {
    /// Validate the entire aggregate
    pub fn validate(&self) -> Result<(), UserRelationshipDomainError> {
        // Rule: User cannot have relationship with themselves
        if self.user_id == self.target_user_id {
            return Err(UserRelationshipDomainError::CannotBefriendSelf);
        }

        // Rule: Message validation
        if let Some(msg) = &self.message {
            if msg.trim().is_empty() {
                return Err(UserRelationshipDomainError::MessageEmpty);
            }
            if msg.len() > 200 {
                return Err(UserRelationshipDomainError::MessageTooLong);
            }
        }

        // Rule: Message should only exist on pending relationships
        if self.message.is_some()
            && !matches!(
                self.relationship_type,
                RelationshipType::PendingIncoming | RelationshipType::PendingOutgoing
            )
        {
            return Err(UserRelationshipDomainError::InvalidStateTransition {
                from: self.relationship_type.as_str().to_owned(),
                to: "message exists on non-pending relationship".to_string(),
            });
        }

        Ok(())
    }
}

// =====================================================
// INTERNAL HELPERS (Private)
// =====================================================

impl UserRelationship {
    /// Update message (only for pending relationships)
    fn update_message(
        &mut self,
        message: Option<String>,
    ) -> Result<(), UserRelationshipDomainError> {
        match self.relationship_type {
            RelationshipType::Friend | RelationshipType::Blocked => {
                // Friends and blocked users cannot have messages
                Err(UserRelationshipDomainError::InvalidStateTransition {
                    from: self.relationship_type.as_str().to_owned(),
                    to: "cannot update message on friend or blocked relationship".to_string(),
                })
            }
            RelationshipType::PendingIncoming | RelationshipType::PendingOutgoing => {
                // Validate message if provided
                if let Some(msg) = &message {
                    if msg.trim().is_empty() {
                        return Err(UserRelationshipDomainError::MessageEmpty);
                    }
                    if msg.len() > 200 {
                        return Err(UserRelationshipDomainError::MessageTooLong);
                    }
                }
                self.message = message;
                self.updated_at = Utc::now();
                Ok(())
            }
        }
    }
}

// =====================================================
// TESTS
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_relationship::erorr::UserRelationshipDomainError;

    #[test]
    fn test_create_friend_request_success() {
        let sender = Uuid::new_v4();
        let receiver = Uuid::new_v4();
        let message = Some("Hey! Let's be friends! 👋".to_string());

        let request = UserRelationship::create_friend_request(&sender, &receiver, message.clone())
            .expect("Should create friend request");

        assert_eq!(*request.user_id(), sender);
        assert_eq!(*request.target_user_id(), receiver);
        assert_eq!(
            *request.relationship_type(),
            RelationshipType::PendingOutgoing
        );
        assert_eq!(request.message(), Some("Hey! Let's be friends! 👋"));
    }

    #[test]
    fn test_create_friend_request_to_self_fails() {
        let user = Uuid::new_v4();
        let result = UserRelationship::create_friend_request(&user, &user, None);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::CannotBefriendSelf
        ));
    }

    #[test]
    fn test_create_friend_request_message_too_long_fails() {
        let sender = Uuid::new_v4();
        let receiver = Uuid::new_v4();
        let long_message = "a".repeat(201);

        let result = UserRelationship::create_friend_request(&sender, &receiver, Some(long_message));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::MessageTooLong
        ));
    }

    #[test]
    fn test_create_friend_request_empty_message_fails() {
        let sender = Uuid::new_v4();
        let receiver = Uuid::new_v4();

        let result =
            UserRelationship::create_friend_request(&sender, &receiver, Some("   ".to_string()));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::MessageEmpty
        ));
    }

    #[test]
    fn test_accept_pending_incoming_success() {
        let mut request = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingIncoming,
            Some("Test".to_string()),
        );

        request.accept().expect("Should accept");

        assert_eq!(*request.relationship_type(), RelationshipType::Friend);
        assert!(request.message().is_none()); // Message cleared
    }

    #[test]
    fn test_accept_non_pending_fails() {
        let mut friend = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::Friend,
            None,
        );

        let result = friend.accept();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::AlreadyFriends
        ));
    }

    #[test]
    fn test_accept_pending_outgoing_fails() {
        let mut request = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingOutgoing,
            Some("Test".to_string()),
        );

        let result = request.accept();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::CannotAcceptNonPendingRequest
        ));
    }

    #[test]
    fn test_create_block_success() {
        let blocker = Uuid::new_v4();
        let blocked = Uuid::new_v4();

        let block = UserRelationship::create_block(blocker, blocked).expect("Should create block");

        assert_eq!(*block.user_id(), blocker);
        assert_eq!(*block.target_user_id(), blocked);
        assert_eq!(*block.relationship_type(), RelationshipType::Blocked);
        assert!(block.message().is_none());
    }

    #[test]
    fn test_create_block_self_fails() {
        let user = Uuid::new_v4();
        let result = UserRelationship::create_block(user, user);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UserRelationshipDomainError::CannotBlockSelf
        ));
    }

    #[test]
    fn test_is_friend() {
        let friend = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::Friend,
            None,
        );

        assert!(friend.is_friend());
        assert!(!friend.is_pending());
        assert!(!friend.is_blocked());
    }

    #[test]
    fn test_is_pending() {
        let incoming = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingIncoming,
            Some("Test".to_string()),
        );

        let outgoing = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingOutgoing,
            Some("Test".to_string()),
        );

        assert!(incoming.is_pending_incoming());
        assert!(incoming.is_pending());
        assert!(outgoing.is_pending_outgoing());
        assert!(outgoing.is_pending());
    }

    #[test]
    fn test_get_other_user_id() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let rel = UserRelationship::new(user1, user2, RelationshipType::Friend, None);

        assert_eq!(rel.get_other_user_id(&user1), Some(user2));
        assert_eq!(rel.get_other_user_id(&user2), Some(user1));
        assert_eq!(rel.get_other_user_id(&Uuid::new_v4()), None);
    }

    #[test]
    fn test_validate_success() {
        let valid = UserRelationship::create_friend_request(
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            Some("Valid message".to_string()),
        )
        .unwrap();

        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_decline_pending_incoming() {
        let mut request = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingIncoming,
            Some("Test".to_string()),
        );

        assert!(request.decline().is_ok());
    }

    #[test]
    fn test_cancel_pending_outgoing() {
        let mut request = UserRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::PendingOutgoing,
            Some("Test".to_string()),
        );

        assert!(request.cancel().is_ok());
    }
}
