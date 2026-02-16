use common::infrastructure::persistence::error::RepositoryError;
use uuid::Uuid;

use crate::domain::user_relationship::UserRelationshipError;

/// Application errors for UserRelationship service
#[derive(Debug, thiserror::Error)]
pub enum UserRelationshipServiceError {
    #[error("Relationship not found")]
    RelationshipNotFound,

    #[error("User not found: {user_id}")]
    UserNotFound { user_id: Uuid },

    #[error("A relationship already exists between these users")]
    RelationshipAlreadyExists,

    #[error("Cannot send friend request: target user has blocked you")]
    BlockedByTarget,

    #[error("Cannot send friend request: you have blocked the target user")]
    BlockedTarget,

    #[error("Domain error: {0}")]
    DomainError(#[from] UserRelationshipError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for UserRelationshipServiceError {
    fn from(error: RepositoryError) -> Self {
        Self::RepositoryError(error.to_string())
    }
}
