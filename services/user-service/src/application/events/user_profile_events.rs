use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// USER PROFILE CREATED
// ============================================

/// Event published when a user profile is created
///
/// Subject: user.profile.created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileCreatedEvent {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

impl UserProfileCreatedEvent {
    pub fn new(user_id: Uuid, username: String, display_name: String) -> Self {
        Self {
            user_id,
            username,
            display_name,
            created_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.profile.created"
    }
}

// ============================================
// USER PROFILE UPDATED
// ============================================

/// Event published when a user profile is updated
///
/// Subject: user.profile.updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdatedEvent {
    pub user_id: Uuid,
    pub updated_fields: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl UserProfileUpdatedEvent {
    pub fn new(user_id: Uuid, updated_fields: Vec<String>) -> Self {
        Self {
            user_id,
            updated_fields,
            updated_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.profile.updated"
    }
}

// ============================================
// USERNAME CHANGED
// ============================================

/// Event published when username is changed
///
/// Subject: user.username.changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameChangedEvent {
    pub user_id: Uuid,
    pub old_username: String,
    pub new_username: String,
    pub changed_at: DateTime<Utc>,
}

impl UsernameChangedEvent {
    pub fn new(user_id: Uuid, old_username: String, new_username: String) -> Self {
        Self {
            user_id,
            old_username,
            new_username,
            changed_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.username.changed"
    }
}

// ============================================
// USER STATUS CHANGED
// ============================================

/// Event published when user status changes
///
/// Subject: user.status.changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatusChangedEvent {
    pub user_id: Uuid,
    pub status: String, // "online", "offline", "idle", "dnd"
    pub changed_at: DateTime<Utc>,
}

impl UserStatusChangedEvent {
    pub fn new(user_id: Uuid, status: String) -> Self {
        Self {
            user_id,
            status,
            changed_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.status.changed"
    }
}

// ============================================
// USER PROFILE DELETED
// ============================================

/// Event published when a user profile is deleted
///
/// Subject: user.profile.deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileDeletedEvent {
    pub user_id: Uuid,
    pub username: String,
    pub deleted_at: DateTime<Utc>,
}

impl UserProfileDeletedEvent {
    pub fn new(user_id: Uuid, username: String) -> Self {
        Self {
            user_id,
            username,
            deleted_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.profile.deleted"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_created_event() {
        let event = UserProfileCreatedEvent::new(
            Uuid::new_v4(),
            "testuser".to_string(),
            "Test User".to_string(),
        );

        assert_eq!(event.username, "testuser");
        assert_eq!(UserProfileCreatedEvent::subject(), "user.profile.created");
    }

    #[test]
    fn test_event_serialization() {
        let event = UsernameChangedEvent::new(
            Uuid::new_v4(),
            "oldname".to_string(),
            "newname".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UsernameChangedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.old_username, deserialized.old_username);
        assert_eq!(event.new_username, deserialized.new_username);
    }
}
