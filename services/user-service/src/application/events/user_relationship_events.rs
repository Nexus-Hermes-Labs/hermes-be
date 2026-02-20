use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Emitted when a friend request is sent
#[derive(Debug, Clone)]
pub struct FriendRequestSentEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl FriendRequestSentEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid, message: String) -> Self {
        Self {
            user_id,
            target_user_id,
            message,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.friend_request.sent.{}", self.user_id)
    }
}

/// Emitted when a friend request is accepted
#[derive(Debug, Clone)]
pub struct FriendRequestAcceptedEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl FriendRequestAcceptedEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid) -> Self {
        Self {
            user_id,
            target_user_id,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.friend_request.accepted.{}", self.user_id)
    }
}

/// Emitted when a friend request is declined
#[derive(Debug, Clone)]
pub struct FriendRequestDeclinedEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl FriendRequestDeclinedEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid) -> Self {
        Self {
            user_id,
            target_user_id,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.friend_request.declined.{}", self.user_id)
    }
}

/// Emitted when a friendship is removed
#[derive(Debug, Clone)]
pub struct FriendRemovedEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl FriendRemovedEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid) -> Self {
        Self {
            user_id,
            target_user_id,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.friend.removed.{}", self.user_id)
    }
}

/// Emitted when a user is blocked
#[derive(Debug, Clone)]
pub struct UserBlockedEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl UserBlockedEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid) -> Self {
        Self {
            user_id,
            target_user_id,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.blocked.{}", self.user_id)
    }
}

/// Emitted when a user is unblocked
#[derive(Debug, Clone)]
pub struct UserUnblockedEvent {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl UserUnblockedEvent {
    pub fn new(user_id: Uuid, target_user_id: Uuid) -> Self {
        Self {
            user_id,
            target_user_id,
            timestamp: Utc::now(),
        }
    }

    pub fn subject(&self) -> String {
        format!("user.relationship.unblocked.{}", self.user_id)
    }
}
