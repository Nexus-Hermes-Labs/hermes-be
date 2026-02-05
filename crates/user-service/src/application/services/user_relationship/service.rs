use super::error::UserRelationshipApplicationError;
use crate::domain::user::repository::UserRepository;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user_relationship::service::UserRelationshipDomainService;
use crate::domain::user_relationship::{
    entity::UserRelationship,
    repository::UserRelationshipRepository,
    valueobject::{RelationshipType, UserRelationshipWithTarget},
};
use common::pagination::{Paginated, PaginationParams};
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

pub struct UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    relationship_repo: Arc<RR>,
    user_repo: Arc<UR>,
}

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    pub fn new(relationship_repo: Arc<RR>, user_repo: Arc<UR>) -> Self {
        Self {
            relationship_repo,
            user_repo,
        }
    }
}

// =====================================================
// FRIEND REQUEST OPERATIONS
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Send a friend request
    ///
    /// # Business Flow
    /// 1. Validate both users exist
    /// 2. Check for existing relationship
    /// 3. Check if blocked (either direction)
    /// 4. Validate privacy settings
    /// 5. Create pending_outgoing request
    /// 6. DB trigger creates pending_incoming for receiver
    #[instrument(
        skip(self),
        fields(sender_id = %sender_id, receiver_username = %receiver_username)
    )]
    pub async fn send_friend_request(
        &self,
        sender_id: Uuid,
        receiver_username: &str,
        message: Option<String>,
    ) -> Result<UserRelationship, UserRelationshipApplicationError> {
        info!("Sending friend request");

        // 1. Fetch sender
        let sender = self
            .user_repo
            .find_by_id(sender_id)
            .await?
            .ok_or(UserRelationshipApplicationError::UserNotFound(sender_id))?;

        // 2. Fetch receiver
        let receiver = self
            .user_repo
            .find_by_username(receiver_username)
            .await?
            .ok_or_else(|| {
                UserRelationshipApplicationError::TargetUserNotFound(receiver_username.to_string())
            })?;

        // 3. Check for existing relationship
        if let Some(existing) = self
            .relationship_repo
            .find_relationship(&sender_id, &receiver.id)
            .await?
        {
            // Convert to appropriate domain error
            return Err(self
                .classify_existing_relationship(&existing, &sender_id)
                .into());
        }

        // 4. Check if blocked (reverse direction)
        if self
            .relationship_repo
            .is_blocked(&receiver.id, &sender_id)
            .await?
        {
            warn!("Friend request blocked: receiver has blocked sender");
            return Err(UserRelationshipDomainError::UserIsBlocked.into());
        }

        // TODO: Check friends of friends (for privacy)
        // let are_friends_of_friends = self
        //     .check_friends_of_friends(&sender_id, receiver.id)
        //     .await?;

        // 5. Domain validation (privacy, business rules)
        UserRelationship::validate_friend_request(&sender.id, &receiver.id)
            .await?;

        // 6. Create friend request (domain logic)
        let friend_request =
            UserRelationship::create_friend_request(&sender_id, *receiver.id, message)?;

        // 7. Save (trigger creates reverse)
        self.relationship_repo.save(&friend_request).await?;

        info!("Friend request sent successfully");

        // TODO: Publish domain event
        // self.event_bus.publish(FriendRequestSentEvent { ... });

        Ok(friend_request)
    }

    /// Accept a friend request
    #[instrument(skip(self), fields(receiver_id = %receiver_id, sender_id = %sender_id))]
    pub async fn accept_friend_request(
        &self,
        receiver_id: Uuid,
        sender_id: Uuid,
    ) -> Result<UserRelationship, UserRelationshipApplicationError> {
        info!("Accepting friend request");

        // Find relationship
        let mut relationship = self
            .relationship_repo
            .find_relationship(&receiver_id, &sender_id)
            .await?
            .ok_or(UserRelationshipApplicationError::FriendRequestNotFound)?;

        // Accept (domain behavior)
        relationship.accept()?;

        // Update (trigger updates reverse)
        self.relationship_repo.update(&relationship).await?;

        info!("Friend request accepted successfully");

        // TODO: Publish event

        Ok(relationship)
    }

    /// Decline a friend request
    #[instrument(skip(self), fields(receiver_id = %receiver_id, sender_id = %sender_id))]
    pub async fn decline_friend_request(
        &self,
        receiver_id: Uuid,
        sender_id: Uuid,
    ) -> Result<(), UserRelationshipApplicationError> {
        info!("Declining friend request");

        // Find relationship
        let relationship = self
            .relationship_repo
            .find_relationship(&receiver_id, &sender_id)
            .await?
            .ok_or(UserRelationshipApplicationError::FriendRequestNotFound)?;

        // Verify it's pending_incoming
        if !relationship.is_pending_incoming() {
            warn!("Cannot decline: not a pending incoming request");
            return Err(UserRelationshipDomainError::CannotDeclineNonPendingRequest.into());
        }

        // Delete (trigger deletes reverse)
        self.relationship_repo
            .delete_relationship(&receiver_id, &sender_id)
            .await?;

        info!("Friend request declined successfully");

        Ok(())
    }

    /// Cancel a sent friend request
    #[instrument(skip(self), fields(sender_id = %sender_id, receiver_id = %receiver_id))]
    pub async fn cancel_friend_request(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<(), UserRelationshipApplicationError> {
        info!("Canceling friend request");

        // Find relationship
        let relationship = self
            .relationship_repo
            .find_relationship(&sender_id, &receiver_id)
            .await?
            .ok_or(UserRelationshipApplicationError::FriendRequestNotFound)?;

        // Verify it's pending_outgoing
        if !relationship.is_pending_outgoing() {
            warn!("Cannot cancel: not a pending outgoing request");
            return Err(UserRelationshipDomainError::NotAuthorized.into());
        }

        // Delete (trigger deletes reverse)
        self.relationship_repo
            .delete_relationship(&sender_id, &receiver_id)
            .await?;

        info!("Friend request canceled successfully");

        Ok(())
    }
}

// =====================================================
// FRIEND OPERATIONS
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Get friends with user details (enriched)
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_friends(
        &self,
        user_id: Uuid,
        params: PaginationParams,
    ) -> Result<Paginated<UserRelationshipWithTarget>, UserRelationshipApplicationError> {
        info!("Fetching friends");

        // 1. Fetch friend relationships
        let friends_page = self
            .relationship_repo
            .find_friends(&user_id, &params)
            .await?;

        // 2. Extract target user IDs
        let target_ids = UserRelationshipDomainService::extract_target_ids(&friends_page.items);

        // 3. Fetch users in bulk
        let users = self.user_repo.find_by_ids(&target_ids).await?;

        // 4. Domain service enriches
        let enriched = UserRelationshipDomainService::enrich_with_targets(friends_page.items, users);

        Ok(Paginated::new(
            enriched,
            friends_page.total,
            params.page,
            params.page_size,
        ))
    }

    /// Remove a friend (unfriend)
    #[instrument(skip(self), fields(user_id = %user_id, friend_id = %friend_id))]
    pub async fn remove_friend(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<(), UserRelationshipApplicationError> {
        info!("Removing friend");

        // Verify they are friends
        if !self
            .relationship_repo
            .are_friends(&user_id, &friend_id)
            .await?
        {
            return Err(UserRelationshipApplicationError::FriendshipNotFound);
        }

        // Delete (trigger deletes both directions)
        self.relationship_repo
            .delete_relationship(&user_id, &friend_id)
            .await?;

        info!("Friend removed successfully");

        // TODO: Publish event

        Ok(())
    }

    /// Count friends
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn count_friends(
        &self,
        user_id: Uuid,
    ) -> Result<i64, UserRelationshipApplicationError> {
        Ok(self.relationship_repo.count_friends(&user_id).await?)
    }
}

// =====================================================
// PENDING REQUEST QUERIES
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Get pending incoming requests with sender details
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_pending_incoming_requests(
        &self,
        user_id: Uuid,
        params: PaginationParams,
    ) -> Result<Paginated<UserRelationshipWithTarget>, UserRelationshipApplicationError> {
        info!("Fetching pending incoming requests");

        let requests_page = self
            .relationship_repo
            .find_pending_incoming(&user_id, &params)
            .await?;

        let sender_ids = UserRelationshipDomainService::extract_target_ids(&requests_page.items);
        let users = self.user_repo.find_by_ids(&sender_ids).await?;
        let enriched = UserRelationshipDomainService::enrich_with_targets(requests_page.items, users);

        Ok(Paginated::new(
            enriched,
            requests_page.total,
            params.page,
            params.page_size,
        ))
    }

    /// Get pending outgoing requests with receiver details
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_pending_outgoing_requests(
        &self,
        user_id: Uuid,
        params: PaginationParams,
    ) -> Result<Paginated<UserRelationshipWithTarget>, UserRelationshipApplicationError> {
        info!("Fetching pending outgoing requests");

        let requests_page = self
            .relationship_repo
            .find_pending_outgoing(&user_id, &params)
            .await?;

        let receiver_ids = UserRelationshipDomainService::extract_target_ids(&requests_page.items);
        let users = self.user_repo.find_by_ids(&receiver_ids).await?;
        let enriched = UserRelationshipDomainService::enrich_with_targets(requests_page.items, users);

        Ok(Paginated::new(
            enriched,
            requests_page.total,
            params.page,
            params.page_size,
        ))
    }

    /// Count pending incoming requests (for notification badge)
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn count_pending_incoming(
        &self,
        user_id: Uuid,
    ) -> Result<i64, UserRelationshipApplicationError> {
        Ok(self
            .relationship_repo
            .count_pending_incoming(&user_id)
            .await?)
    }

    /// Count pending outgoing requests
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn count_pending_outgoing(
        &self,
        user_id: Uuid,
    ) -> Result<i64, UserRelationshipApplicationError> {
        Ok(self
            .relationship_repo
            .count_pending_outgoing(&user_id)
            .await?)
    }
}

// =====================================================
// BLOCK OPERATIONS
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Block a user
    #[instrument(
        skip(self),
        fields(blocker_id = %blocker_id, blocked_username = %blocked_username)
    )]
    pub async fn block_user(
        &self,
        blocker_id: Uuid,
        blocked_username: &str,
    ) -> Result<UserRelationship, UserRelationshipApplicationError> {
        info!("Blocking user");

        // 1. Fetch blocker
        let blocker = self
            .user_repo
            .find_by_id(blocker_id)
            .await?
            .ok_or(UserRelationshipApplicationError::UserNotFound(blocker_id))?;

        // 2. Fetch blocked user
        let blocked = self
            .user_repo
            .find_by_username(blocked_username)
            .await?
            .ok_or_else(|| {
                UserRelationshipApplicationError::TargetUserNotFound(blocked_username.to_string())
            })?;

        // 3. Domain validation
        UserRelationship::validate_block(&blocker.id, &blocked.id).await?;

        // 4. Remove any existing relationship first
        let _ = self
            .relationship_repo
            .delete_relationship(&blocker_id, blocked.id.as_ref())
            .await;

        // 5. Create block
        let block = UserRelationship::create_block(blocker_id, *blocked.id)?;

        // 6. Save (no reverse relationship for blocks)
        self.relationship_repo.save(&block).await?;

        info!("User blocked successfully");

        // TODO: Publish event

        Ok(block)
    }

    /// Unblock a user
    #[instrument(skip(self), fields(blocker_id = %blocker_id, blocked_id = %blocked_id))]
    pub async fn unblock_user(
        &self,
        blocker_id: Uuid,
        blocked_id: Uuid,
    ) -> Result<(), UserRelationshipApplicationError> {
        info!("Unblocking user");

        // Verify block exists
        if !self
            .relationship_repo
            .is_blocked(&blocker_id, &blocked_id)
            .await?
        {
            return Err(UserRelationshipApplicationError::RelationshipNotFound);
        }

        // Delete block
        self.relationship_repo
            .delete_relationship(&blocker_id, &blocked_id)
            .await?;

        info!("User unblocked successfully");

        Ok(())
    }

    /// Get blocked users with details
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_blocked_users(
        &self,
        user_id: Uuid,
        params: PaginationParams,
    ) -> Result<Paginated<UserRelationshipWithTarget>, UserRelationshipApplicationError> {
        info!("Fetching blocked users");

        let blocked_page = self
            .relationship_repo
            .find_blocked(&user_id, &params)
            .await?;

        let blocked_ids = UserRelationshipDomainService::extract_target_ids(&blocked_page.items);
        let users = self.user_repo.find_by_ids(&blocked_ids).await?;
        let enriched = UserRelationshipDomainService::enrich_with_targets(blocked_page.items, users);

        Ok(Paginated::new(
            enriched,
            blocked_page.total,
            params.page,
            params.page_size,
        ))
    }

    /// Count blocked users
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn count_blocked(
        &self,
        user_id: Uuid,
    ) -> Result<i64, UserRelationshipApplicationError> {
        Ok(self.relationship_repo.count_blocked(&user_id).await?)
    }
}

// =====================================================
// RELATIONSHIP STATUS QUERIES
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Check if two users are friends
    #[instrument(skip(self), fields(user_id = %user_id, other_user_id = %other_user_id))]
    pub async fn are_friends(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<bool, UserRelationshipApplicationError> {
        Ok(self
            .relationship_repo
            .are_friends(&user_id, &other_user_id)
            .await?)
    }

    /// Check if user has blocked target
    #[instrument(skip(self), fields(user_id = %user_id, target_user_id = %target_user_id))]
    pub async fn is_blocked(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<bool, UserRelationshipApplicationError> {
        Ok(self
            .relationship_repo
            .is_blocked(&user_id, &target_user_id)
            .await?)
    }

    /// Get comprehensive relationship status between two users
    #[instrument(skip(self), fields(user_id = %user_id, other_user_id = %other_user_id))]
    pub async fn get_relationship_status(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<RelationshipStatus, UserRelationshipApplicationError> {
        // Check if friends
        if self
            .relationship_repo
            .are_friends(&user_id, &other_user_id)
            .await?
        {
            return Ok(RelationshipStatus::Friends);
        }

        // Check if blocked
        if self
            .relationship_repo
            .is_blocked(&user_id, &other_user_id)
            .await?
        {
            return Ok(RelationshipStatus::YouBlockedThem);
        }

        if self
            .relationship_repo
            .is_blocked(&other_user_id, &user_id)
            .await?
        {
            return Ok(RelationshipStatus::TheyBlockedYou);
        }

        // Check for pending requests
        if let Some(rel) = self
            .relationship_repo
            .find_relationship(&user_id, &other_user_id)
            .await?
        {
            return Ok(match rel.relationship_type() {
                RelationshipType::PendingOutgoing => RelationshipStatus::PendingOutgoing,
                RelationshipType::PendingIncoming => RelationshipStatus::PendingIncoming,
                _ => RelationshipStatus::None,
            });
        }

        Ok(RelationshipStatus::None)
    }
}

// =====================================================
// HELPER METHODS (Private)
// =====================================================

impl<RR, UR> UserRelationshipApplicationService<RR, UR>
where
    RR: UserRelationshipRepository,
    UR: UserRepository,
{
    /// Classify existing relationship to appropriate domain error
    fn classify_existing_relationship(
        &self,
        relationship: &UserRelationship,
        sender_id: &Uuid,
    ) -> UserRelationshipDomainError {
        match relationship.relationship_type() {
            RelationshipType::Friend => UserRelationshipDomainError::AlreadyFriends,
            RelationshipType::PendingOutgoing if relationship.user_id() == sender_id => {
                UserRelationshipDomainError::PendingRequestAlreadySent
            }
            RelationshipType::PendingIncoming => {
                UserRelationshipDomainError::PendingRequestAlreadyReceived
            }
            RelationshipType::Blocked if relationship.user_id() == sender_id => {
                UserRelationshipDomainError::YouHaveBlockedUser
            }
            _ => UserRelationshipDomainError::RelationshipAlreadyExists,
        }
    }

    /// Check if users are friends of friends (for privacy validation)
    async fn check_friends_of_friends(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<bool, UserRelationshipApplicationError> {
        // Get user's friends
        let user_friends = self
            .relationship_repo
            .find_friends(user_id, &PaginationParams::default())
            .await?;

        // Get other user's friends
        let other_friends = self
            .relationship_repo
            .find_friends(other_user_id, &PaginationParams::default())
            .await?;

        // Check for mutual friends
        let user_friend_ids: std::collections::HashSet<_> = user_friends
            .items
            .iter()
            .map(|rel| *rel.target_user_id())
            .collect();

        let has_mutual = other_friends
            .items
            .iter()
            .any(|rel| user_friend_ids.contains(rel.target_user_id()));

        Ok(has_mutual)
    }
}

// =====================================================
// SUPPORTING TYPES
// =====================================================

/// Comprehensive relationship status between two users
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelationshipStatus {
    None,
    Friends,
    PendingOutgoing, // You sent a request
    PendingIncoming, // They sent a request
    YouBlockedThem,
    TheyBlockedYou,
}

#[cfg(test)]
mod tests {
    // TODO: Add unit tests with mock repositories
}
