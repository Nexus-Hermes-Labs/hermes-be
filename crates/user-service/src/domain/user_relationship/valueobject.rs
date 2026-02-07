use uuid::Uuid;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user::valueobject::UserSnapshot;
use super::entity::UserRelationship;

/// Represents the state of a relationship between two users.
///
/// This enum is part of the domain layer and defines the
/// lifecycle stages of a user relationship.
///
/// Stored values are string-based to simplify persistence,
/// serialization, and interoperability across boundaries.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    /// Users are mutually connected.
    Friend,

    /// One user has blocked the other.
    Blocked,

    /// A pending request received by the current user.
    PendingIncoming,

    /// A pending request sent by the current user.
    PendingOutgoing,
}

impl RelationshipType {
    /// Returns the canonical string representation
    /// used for persistence and external communication.
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Friend => "friend",
            RelationshipType::Blocked => "blocked",
            RelationshipType::PendingIncoming => "pending_incoming",
            RelationshipType::PendingOutgoing => "pending_outgoing",
        }
    }
}

impl TryFrom<&str> for RelationshipType {
    type Error = UserRelationshipDomainError;

    /// Attempts to convert a string into a `RelationshipType`.
    ///
    /// Returns a domain error if the value is unknown.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "friend" => RelationshipType::Friend,
            "blocked" => RelationshipType::Blocked,
            "pending_incoming" => RelationshipType::PendingIncoming,
            "pending_outgoing" => RelationshipType::PendingOutgoing,
            _ => {
                return Err(
                    UserRelationshipDomainError::InvalidRelationshipType(
                        value.into()
                    )
                )
            }
        })
    }
}

/// A DOMAIN VALUE OBJECT representing a user relationship
/// enriched with target user snapshot data.
///
/// Typically used in read/query scenarios where both the
/// relationship state and lightweight user info are needed.
#[derive(Debug, Clone)]
pub struct UserRelationshipWithTarget {
    /// Core relationship domain entity
    pub relationship: UserRelationship,

    /// Snapshot data of the related (target) user
    pub target_user: UserSnapshot,
}

impl UserRelationshipWithTarget {
    /// Creates a new combined relationship value object.
    pub fn new(
        relationship: UserRelationship,
        target_user: UserSnapshot,
    ) -> Self {
        Self {
            relationship,
            target_user,
        }
    }

    /// Returns the unique identifier of the relationship.
    #[inline]
    pub fn id(&self) -> &Uuid {
        self.relationship.id()
    }

    /// Returns true if the relationship is a friendship.
    #[inline]
    pub fn is_friend(&self) -> bool {
        self.relationship.is_friend()
    }

    /// Returns true if the relationship is in any pending state.
    #[inline]
    pub fn is_pending(&self) -> bool {
        self.relationship.is_pending_incoming()
            || self.relationship.is_pending_outgoing()
    }

    /// Returns the optional relationship message if present.
    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.relationship.message()
    }
}
