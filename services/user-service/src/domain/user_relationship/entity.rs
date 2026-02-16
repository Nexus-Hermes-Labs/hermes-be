use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::UserRelationshipError;
use super::valueobject::RelationshipType;

/// Represents a directional relationship between two users.
///
/// This aggregate models friend requests, friendships, and blocks.
///
/// ⚠️ Important:
/// - Relationships are directional.
/// - For friend requests, two records are typically created:
///     - `PendingOutgoing`  (request sender → receiver)
///     - `PendingIncoming`  (receiver ← sender)
/// - When a friendship is established, both sides become `Friend`.
/// - When a request is declined, both pending records are removed
///   at the application layer within a transaction.
///
/// This entity does NOT handle persistence concerns.
/// Deletion and cross-record consistency must be handled
/// in the application layer.
#[derive(Debug, Clone)]
pub struct UserRelationship {
    // ============================================
    // IDENTITY
    // ============================================

    /// Unique identifier of this relationship record.
    id: Uuid,

    /// The owner of this relationship entry.
    /// (The user who "sees" this relationship)
    user_id: Uuid,

    /// The other user involved in this relationship.
    target_user_id: Uuid,

    // ============================================
    // STATE
    // ============================================

    /// Current type of the relationship.
    /// Determines behavior and allowed transitions.
    relationship_type: RelationshipType,

    /// Optional message attached to a friend request.
    /// Only present for pending requests.
    message: Option<String>,

    // ============================================
    // METADATA
    // ============================================

    /// Timestamp when this relationship was created.
    created_at: DateTime<Utc>,

    /// Timestamp of the last state change.
    /// Updated on transitions like `accept`.
    updated_at: DateTime<Utc>,
}

impl UserRelationship {
    // ============================================
    // CONSTRUCTION
    // ============================================

    pub fn create_request(
        user_id: Uuid,
        target_user_id: Uuid,
        message: String,
    ) -> Result<Self, UserRelationshipError> {
        if user_id == target_user_id {
            return Err(UserRelationshipError::SelfRelationship);
        }
        Self::validate_message(Some(&message))?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            target_user_id,
            relationship_type: RelationshipType::PendingOutgoing,
            message: Some(message),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn create_block(
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<Self, UserRelationshipError> {
        if user_id == target_user_id {
            return Err(UserRelationshipError::SelfRelationship);
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            target_user_id,
            relationship_type: RelationshipType::Blocked,
            message: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstructs a relationship from persisted data.
    ///
    /// This bypasses validation and assumes the data
    /// has already satisfied all domain invariants.
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
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

    // ============================================
    // GETTERS
    // ============================================

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn target_user_id(&self) -> Uuid {
        self.target_user_id
    }

    pub fn relationship_type(&self) -> RelationshipType {
        self.relationship_type
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ============================================
    // STATE CHECKS
    // ============================================

    pub fn is_friend(&self) -> bool {
        self.relationship_type == RelationshipType::Friend
    }

    pub fn is_blocked(&self) -> bool {
        self.relationship_type == RelationshipType::Blocked
    }

    pub fn is_pending(&self) -> bool {
        matches!(
            self.relationship_type,
            RelationshipType::PendingIncoming | RelationshipType::PendingOutgoing
        )
    }

    // ============================================
    // DOMAIN LOGIC
    // ============================================

    /// Accept a friend request.
    /// This can only be done on a `PendingIncoming` relationship.
    /// Transitions the relationship to `Friend`.
    /// Clears any request message upon acceptance.
    pub fn accept(&mut self) -> Result<(), UserRelationshipError> {
        if self.relationship_type != RelationshipType::PendingIncoming {
            return Err(UserRelationshipError::CannotAccept);
        }

        self.relationship_type = RelationshipType::Friend;
        self.message = None; // Message is cleared on acceptance
        self.updated_at = Utc::now();

        Ok(())
    }

    // TODO: will handle this on application layer with transaction!
    /// Validates that this relationship can be declined.
    /// This can only be done on a `PendingIncoming` relationship.
    pub fn decline(&mut self) -> Result<(), UserRelationshipError> {
        if self.relationship_type != RelationshipType::PendingIncoming {
            return Err(UserRelationshipError::CannotDecline);
        }

        Ok(())
    }

    /// Block the target user.
    /// This can be done from any state.
    /// Transitions the relationship to `Blocked`.
    /// Clears any request message upon blocking.
    pub fn block(&mut self) -> Result<(), UserRelationshipError> {
        if self.is_blocked() {
            return Err(UserRelationshipError::AlreadyBlocked);
        }

        self.relationship_type = RelationshipType::Blocked;
        self.message = None; // Message is cleared on block
        self.updated_at = Utc::now();

        Ok(())
    }

    // ============================================
    // HELPERS
    // ============================================

    /// Ensures that a request message does not exceed 200 characters.
    fn validate_message(message: Option<&String>) -> Result<(), UserRelationshipError> {
        if let Some(msg) = message {
            if msg.len() > 200 {
                return Err(UserRelationshipError::MessageTooLong);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ids() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    #[test]
    fn test_create_request() {
        let (user_id, target_user_id) = create_test_ids();
        let message = "Hey, let's be friends!".to_string();
        let relationship =
            UserRelationship::create_request(user_id, target_user_id, message.clone()).unwrap();

        assert_eq!(relationship.user_id(), user_id);
        assert_eq!(relationship.target_user_id(), target_user_id);
        assert_eq!(
            relationship.relationship_type(),
            RelationshipType::PendingOutgoing
        );
        assert_eq!(relationship.message(), Some(message.as_str()));
        assert!(relationship.is_pending());
    }

    #[test]
    fn test_create_request_self_relationship_fails() {
        let user_id = Uuid::new_v4();
        let message = "msg".to_string();
        let result = UserRelationship::create_request(user_id, user_id, message);
        assert!(matches!(
            result,
            Err(UserRelationshipError::SelfRelationship)
        ));
    }

    #[test]
    fn test_create_request_message_too_long_fails() {
        let (user_id, target_user_id) = create_test_ids();
        let long_message = "a".repeat(201);
        let result = UserRelationship::create_request(user_id, target_user_id, long_message);
        assert!(matches!(result, Err(UserRelationshipError::MessageTooLong)));
    }

    #[test]
    fn test_create_block() {
        let (user_id, target_user_id) = create_test_ids();
        let relationship = UserRelationship::create_block(user_id, target_user_id).unwrap();

        assert_eq!(relationship.user_id(), user_id);
        assert_eq!(relationship.target_user_id(), target_user_id);
        assert_eq!(relationship.relationship_type(), RelationshipType::Blocked);
        assert_eq!(relationship.message(), None);
        assert!(relationship.is_blocked());
    }

    #[test]
    fn test_create_block_self_relationship_fails() {
        let user_id = Uuid::new_v4();
        let result = UserRelationship::create_block(user_id, user_id);
        assert!(matches!(
            result,
            Err(UserRelationshipError::SelfRelationship)
        ));
    }

    #[test]
    fn test_accept_friend_request() {
        let (user_id, target_user_id) = create_test_ids();
        let mut relationship = UserRelationship::from_persisted(
            Uuid::new_v4(),
            user_id,
            target_user_id,
            RelationshipType::PendingIncoming,
            Some("pls accept".to_string()),
            Utc::now(),
            Utc::now(),
        );

        relationship.accept().unwrap();

        assert!(relationship.is_friend());
        assert!(!relationship.is_pending());
        assert_eq!(relationship.message(), None);
    }

    #[test]
    fn test_accept_non_pending_request_fails() {
        let (user_id, target_user_id) = create_test_ids();
        let mut relationship = UserRelationship::from_persisted(
            Uuid::new_v4(),
            user_id,
            target_user_id,
            RelationshipType::Friend,
            None,
            Utc::now(),
            Utc::now(),
        );

        let result = relationship.accept();
        assert!(matches!(result, Err(UserRelationshipError::CannotAccept)));
    }

    #[test]
    fn test_block_user() {
        let (user_id, target_user_id) = create_test_ids();
        let mut relationship = UserRelationship::from_persisted(
            Uuid::new_v4(),
            user_id,
            target_user_id,
            RelationshipType::Friend,
            None,
            Utc::now(),
            Utc::now(),
        );

        relationship.block().unwrap();
        assert!(relationship.is_blocked());
        assert!(!relationship.is_friend());
    }

    #[test]
    fn test_block_already_blocked_user_fails() {
        let (user_id, target_user_id) = create_test_ids();
        let mut relationship = UserRelationship::from_persisted(
            Uuid::new_v4(),
            user_id,
            target_user_id,
            RelationshipType::Blocked,
            None,
            Utc::now(),
            Utc::now(),
        );

        let result = relationship.block();
        assert!(matches!(result, Err(UserRelationshipError::AlreadyBlocked)));
    }
}
