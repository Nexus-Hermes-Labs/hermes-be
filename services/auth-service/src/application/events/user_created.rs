use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use common::domain::event::IntoEventEnvelope;

/// Event published when a new user_profile successfully registers
///
/// This event is published AFTER the user_profile profile is created in user_profile-service.
/// Other services can subscribe to this event to:
/// - Send welcome email (notification-service)
/// - Track user_profile acquisition (analytics-service)
/// - Initialize user_profile preferences (settings-service)
/// - Create default data (content-service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    /// User ID (same as auth credential ID)
    pub user_id: Uuid,

    /// User's email address
    pub email: String,

    /// Username (unique with discriminator)
    pub username: String,

    /// User's display name
    pub display_name: String,

    /// When the user_profile was created
    pub timestamp: DateTime<Utc>,

    /// Email verification status at creation
    pub email_verified: bool,
}

impl UserCreatedEvent {
    pub fn new(
        user_id: Uuid,
        email: String,
        username: String,
        display_name: String,
    ) -> Self {
        Self {
            user_id,
            email,
            username,
            display_name,
            timestamp: Utc::now(),
            email_verified: false, // Always false on registration
        }
    }

    /// Create event with custom timestamp (for testing)
    #[cfg(test)]
    pub fn new_with_timestamp(
        user_id: Uuid,
        email: String,
        username: String,
        display_name: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            email,
            username,
            display_name,
            timestamp,
            email_verified: false,
        }
    }
}

impl IntoEventEnvelope for UserCreatedEvent {
    fn event_type(&self) -> &'static str {
        "user_profile.created"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_created_event() {
        let user_id = Uuid::new_v4();
        let event = UserCreatedEvent::new(
            user_id,
            "test@example.com".to_string(),
            "testuser".to_string(),
            "Test User".to_string(),
        );

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.email, "test@example.com");
        assert_eq!(event.username, "testuser");
        assert_eq!(event.display_name, "Test User");
        assert!(!event.email_verified);
        assert_eq!(event.event_type(), "user_profile.created");
    }

    #[test]
    fn test_into_envelope() {
        let user_id = Uuid::new_v4();
        let event = UserCreatedEvent::new(
            user_id,
            "test@example.com".to_string(),
            "testuser".to_string(),
            "Test User".to_string(),
        );

        let envelope = event.into_envelope("auth-service");

        assert_eq!(envelope.event_type, "user_profile.created");
        assert_eq!(envelope.aggregate_id, user_id);
        assert_eq!(envelope.source_service, "auth-service");
        assert_eq!(envelope.payload.user_id, user_id);
    }
}
