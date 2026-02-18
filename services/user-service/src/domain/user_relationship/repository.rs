use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use uuid::Uuid;

use super::entity::UserRelationship;
use super::valueobject::RelationshipType;

/// User relationship repository
#[async_trait]
pub trait UserRelationshipRepository:
    Repository<UserRelationship, Uuid, Error = RepositoryError> + Send + Sync
{
    // Inherits from Repository<UserRelationship, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<UserRelationship>>
    // - save(entity: &UserRelationship) -> Result<()>
    // - update(entity: &UserRelationship) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>

    // ============================================
    // PAIR LOOKUP
    // ============================================

    /// Find the relationship record owned by `user_id` targeting `target_user_id`.
    async fn find_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<Option<UserRelationship>, Self::Error>;

    /// Check whether a relationship record exists between two users
    /// (in the `user_id → target_user_id` direction).
    async fn exists_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<bool, Self::Error>;

    /// Check whether either user has blocked the other.
    async fn is_blocked_bidirectional(
        &self,
        user_a: Uuid,
        user_b: Uuid,
    ) -> Result<bool, Self::Error>;

    /// Delete the relationship record owned by `user_id` targeting `target_user_id`.
    ///
    /// The `sync_bidirectional_relationship` trigger automatically removes
    /// the reverse record as well (except when the reverse is `blocked`).
    /// This means:
    /// - Decline/unfriend: deleting one side removes both sides via trigger.
    /// - Unblock: deleting a `blocked` record only removes the blocker's side
    ///   (the trigger skips reverse deletion for blocks).
    async fn delete_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), Self::Error>;

    // ============================================
    // FILTERED QUERIES
    // ============================================

    /// Find all relationships for a user, optionally filtered by type.
    async fn find_all_by_user(
        &self,
        user_id: Uuid,
        relationship_type: Option<RelationshipType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, Self::Error>;

    /// Count relationships for a user, optionally filtered by type.
    async fn count_by_user(
        &self,
        user_id: Uuid,
        relationship_type: Option<RelationshipType>,
    ) -> Result<i64, Self::Error>;
}
