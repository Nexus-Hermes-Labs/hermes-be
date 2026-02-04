use crate::domain::user::User;
use crate::domain::user_relationship::entity::UserRelationship;
use async_trait::async_trait;
use common::pagination::{Paginated, PaginationParams};
use common::Repository;
use uuid::Uuid;

/// User relationship repository trait
#[async_trait]
pub trait UserRelationshipRepository: Repository<UserRelationship, Uuid> + Send + Sync {
    // =====================================================
    // SPECIFIC RELATIONSHIP QUERIES
    // =====================================================
    
    /// Find specific relationship between two users (from user_id perspective)
    async fn find_relationship(
        &self,
        user_id: &Uuid,
        target_user_id: &Uuid,
    ) -> Result<Option<UserRelationship>, Self::Error>;
    
    // =====================================================
    // FRIEND QUERIES
    // =====================================================
    
    /// Get all friends of a user (paginated)
    async fn find_friends(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error>;
    
    /// Count total friends
    async fn count_friends(&self, user_id: &Uuid) -> Result<i64, Self::Error>;
    
    // =====================================================
    // PENDING REQUEST QUERIES
    // =====================================================
    
    /// Get pending incoming requests (received)
    async fn find_pending_incoming(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error>;
    
    /// Get pending outgoing requests (sent)
    async fn find_pending_outgoing(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error>;
    
    /// Count pending incoming requests (for notification badge)
    async fn count_pending_incoming(&self, user_id: &Uuid) -> Result<i64, Self::Error>;
    
    /// Count pending outgoing requests (optional - for stats)
    async fn count_pending_outgoing(&self, user_id: &Uuid) -> Result<i64, Self::Error>;
    
    // =====================================================
    // BLOCK QUERIES
    // =====================================================
    
    /// Get all blocked users (paginated)
    async fn find_blocked(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error>;
    
    /// Count blocked users (optional - for stats)
    async fn count_blocked(&self, user_id: &Uuid) -> Result<i64, Self::Error>;
    
    // =====================================================
    // EXISTENCE & RELATIONSHIP CHECKS (Performance Critical)
    // =====================================================
    
    /// Check if two users are friends (fast boolean check)
    /// Used for: Privacy checks, DM permissions
    async fn are_friends(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<bool, Self::Error>;
    
    /// Check if user has blocked target (fast boolean check)
    /// Used for: Interaction blocking, privacy
    async fn is_blocked(
        &self,
        blocker_id: &Uuid,
        blocked_id: &Uuid,
    ) -> Result<bool, Self::Error>;
    
    /// Check if ANY relationship exists between users (any type)
    /// Used for: Preventing duplicate requests
    async fn relationship_exists(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<bool, Self::Error>;
    
    // =====================================================
    // DELETE OPERATIONS
    // =====================================================
    
    /// Delete relationship between two users (both directions)
    /// Trigger will handle reverse relationship deletion
    /// Used for: Unfriend, decline request, cancel request
    async fn delete_relationship(
        &self,
        user_id: &Uuid,
        target_user_id: &Uuid,
    ) -> Result<(), Self::Error>;
}