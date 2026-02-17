use std::sync::Arc;
use uuid::Uuid;

use crate::domain::user_privacy::{FriendRequestPrivacy, UserPrivacyRepository};
use crate::domain::user_relationship::{
    RelationshipType, UserRelationship, UserRelationshipRepository,
};
use crate::application::services::UserPrivacyService;

use super::error::UserRelationshipServiceError;

/// User Relationship Application Service
///
/// Orchestrates friend requests, friendships, and blocks.
///
/// ## Bidirectional Trigger
/// The database has a `sync_bidirectional_relationship` trigger that automatically
/// manages the reverse side of relationships:
/// - **save** a `PendingOutgoing` → trigger creates `PendingIncoming` for the target.
/// - **update** to `Friend` (accept) → trigger syncs the reverse to `Friend`.
/// - **delete** a non-blocked record → trigger removes the reverse record.
/// - **Blocks are one-directional** — the trigger does NOT create a reverse for blocks.
///
/// This means the service only needs to operate on one side; the trigger handles the rest.
pub struct UserRelationshipService {
    repository: Arc<dyn UserRelationshipRepository>,
    privacy_service: Arc<UserPrivacyService>,
}

impl UserRelationshipService {
    pub fn new(
        repository: Arc<dyn UserRelationshipRepository>,
        privacy_service: Arc<UserPrivacyService>,
    ) -> Self {
        Self {
            repository,
            privacy_service,
        }
    }

    // ============================================
    // FRIEND REQUEST MANAGEMENT
    // ============================================

    /// Send a friend request from `user_id` to `target_user_id`.
    ///
    /// Creates a `PendingOutgoing` record for the sender.
    /// The DB trigger automatically creates a `PendingIncoming` for the receiver.
    pub async fn send_friend_request(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
        message: String,
    ) -> Result<UserRelationship, UserRelationshipServiceError> {
        // Check privacy settings of the target user
        let privacy_settings = self
            .privacy_service
            .get_privacy_settings(target_user_id)
            .await
            .map_err(|_| UserRelationshipServiceError::CannotSendFriendRequest)?;

        match privacy_settings.allow_friend_requests_from() {
            FriendRequestPrivacy::None => {
                return Err(UserRelationshipServiceError::CannotSendFriendRequest);
            }
            FriendRequestPrivacy::FriendsOfFriends => {
                // TODO: Implement logic to check for mutual friends
            }
            FriendRequestPrivacy::Everyone => {
                // Allow the request
            }
        }

        // Check if target has blocked the sender
        if self
            .repository
            .exists_by_user_and_target(target_user_id, user_id)
            .await?
        {
            let target_rel = self
                .repository
                .find_by_user_and_target(target_user_id, user_id)
                .await?;
            if target_rel
                .as_ref()
                .is_some_and(|r| r.relationship_type() == RelationshipType::Blocked)
            {
                return Err(UserRelationshipServiceError::BlockedByTarget);
            }
        }

        // Check if sender has blocked the target
        if let Some(existing) = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
        {
            if existing.relationship_type() == RelationshipType::Blocked {
                return Err(UserRelationshipServiceError::BlockedTarget);
            }
            return Err(UserRelationshipServiceError::RelationshipAlreadyExists);
        }

        let relationship = UserRelationship::create_request(user_id, target_user_id, message)
            .map_err(UserRelationshipServiceError::DomainError)?;

        self.repository
            .save(&relationship)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;

        Ok(relationship)
    }

    /// Accept an incoming friend request.
    ///
    /// Loads the receiver's `PendingIncoming` record, transitions it to `Friend`,
    /// and persists it. The DB trigger syncs the sender's side to `Friend` as well.
    pub async fn accept_friend_request(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<UserRelationship, UserRelationshipServiceError> {
        let mut relationship = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
            .ok_or(UserRelationshipServiceError::RelationshipNotFound)?;

        relationship
            .accept()
            .map_err(UserRelationshipServiceError::DomainError)?;

        self.repository
            .update(&relationship)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;

        Ok(relationship)
    }

    /// Decline an incoming friend request.
    ///
    /// Validates the state via domain logic, then deletes the receiver's record.
    /// The DB trigger removes the sender's side automatically.
    pub async fn decline_friend_request(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), UserRelationshipServiceError> {
        let mut relationship = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
            .ok_or(UserRelationshipServiceError::RelationshipNotFound)?;

        relationship
            .decline()
            .map_err(UserRelationshipServiceError::DomainError)?;

        self.repository
            .delete_by_user_and_target(user_id, target_user_id)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    // ============================================
    // FRIENDSHIP MANAGEMENT
    // ============================================

    /// Remove an existing friendship.
    ///
    /// Deletes the caller's side. The DB trigger removes the other side.
    pub async fn remove_friend(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), UserRelationshipServiceError> {
        let relationship = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
            .ok_or(UserRelationshipServiceError::RelationshipNotFound)?;

        if !relationship.is_friend() {
            return Err(UserRelationshipServiceError::RelationshipNotFound);
        }

        self.repository
            .delete_by_user_and_target(user_id, target_user_id)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    // ============================================
    // BLOCK MANAGEMENT
    // ============================================

    /// Block a user.
    ///
    /// If an existing relationship exists (friend/pending), it is removed first,
    /// then a new `Blocked` record is created. Blocks are one-directional —
    /// the trigger does NOT create a reverse record.
    pub async fn block_user(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<UserRelationship, UserRelationshipServiceError> {
        if let Some(mut relationship) = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
        {
            // Relationship exists, transition it to blocked
            relationship
                .block()
                .map_err(UserRelationshipServiceError::DomainError)?;
            self.repository
                .update(&relationship)
                .await
                .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;
            Ok(relationship)
        } else {
            // No relationship, create a new block
            let block = UserRelationship::create_block(user_id, target_user_id)
                .map_err(UserRelationshipServiceError::DomainError)?;
            self.repository
                .save(&block)
                .await
                .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;
            Ok(block)
        }
    }

    /// Unblock a user.
    ///
    /// Deletes the blocker's `Blocked` record. Since blocks are one-directional,
    /// the trigger does not cascade.
    pub async fn unblock_user(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), UserRelationshipServiceError> {
        let relationship = self
            .repository
            .find_by_user_and_target(user_id, target_user_id)
            .await?
            .ok_or(UserRelationshipServiceError::RelationshipNotFound)?;

        if !relationship.is_blocked() {
            return Err(UserRelationshipServiceError::RelationshipNotFound);
        }

        self.repository
            .delete_by_user_and_target(user_id, target_user_id)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    // ============================================
    // QUERIES
    // ============================================

    /// Get all friends for a user (paginated).
    pub async fn get_friends(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, UserRelationshipServiceError> {
        self.repository
            .find_all_by_user(user_id, Some(RelationshipType::Friend), limit, offset)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }

    /// Get all incoming friend requests for a user (paginated).
    pub async fn get_incoming_requests(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, UserRelationshipServiceError> {
        self.repository
            .find_all_by_user(
                user_id,
                Some(RelationshipType::PendingIncoming),
                limit,
                offset,
            )
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }

    /// Get all outgoing friend requests for a user (paginated).
    pub async fn get_outgoing_requests(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, UserRelationshipServiceError> {
        self.repository
            .find_all_by_user(
                user_id,
                Some(RelationshipType::PendingOutgoing),
                limit,
                offset,
            )
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }

    /// Get all blocked users for a user (paginated).
    pub async fn get_blocked_users(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, UserRelationshipServiceError> {
        self.repository
            .find_all_by_user(user_id, Some(RelationshipType::Blocked), limit, offset)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }

    /// Get the relationship between two specific users (from caller's perspective).
    pub async fn get_relationship(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<Option<UserRelationship>, UserRelationshipServiceError> {
        self.repository
            .find_by_user_and_target(user_id, target_user_id)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }

    /// Count relationships for a user, optionally filtered by type.
    pub async fn count_relationships(
        &self,
        user_id: Uuid,
        relationship_type: Option<RelationshipType>,
    ) -> Result<i64, UserRelationshipServiceError> {
        self.repository
            .count_by_user(user_id, relationship_type)
            .await
            .map_err(|e| UserRelationshipServiceError::RepositoryError(e.to_string()))
    }
}
