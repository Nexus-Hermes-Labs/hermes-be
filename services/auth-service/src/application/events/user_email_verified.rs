use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use common::domain::event::IntoEventEnvelope;

/// Event published when user_profile verifies their email address
///
/// This event is published when a user_profile clicks the verification link.
/// Other services can subscribe to this event to:
/// - Send confirmation email (notification-service)
/// - Unlock features (feature-service)
/// - Track verification rate (analytics-service)
/// - Update user_profile status (user_profile-service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmailVerifiedEvent {
    /// User ID
    pub user_id: Uuid,

    /// Verified email address
    pub email: String,

    /// When the email was verified
    pub verified_at: DateTime<Utc>,
}

impl UserEmailVerifiedEvent {
    pub fn new(user_id: Uuid, email: String) -> Self {
        Self {
            user_id,
            email,
            verified_at: Utc::now(),
        }
    }

    /// Create event with custom timestamp (for testing)
    #[cfg(test)]
    pub fn new_with_timestamp(
        user_id: Uuid,
        email: String,
        verified_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            email,
            verified_at,
        }
    }
}

impl IntoEventEnvelope for UserEmailVerifiedEvent {
    fn event_type(&self) -> &'static str {
        "user_profile.email.verified"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_verified_event() {
        let user_id = Uuid::new_v4();
        let event = UserEmailVerifiedEvent::new(user_id, "test@example.com".to_string());

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.email, "test@example.com");
        assert_eq!(event.event_type(), "user_profile.email.verified");
    }

    #[test]
    fn test_into_envelope() {
        let user_id = Uuid::new_v4();
        let event = UserEmailVerifiedEvent::new(user_id, "test@example.com".to_string());
        let envelope = event.into_envelope("auth-service");

        assert_eq!(envelope.event_type, "user_profile.email.verified");
        assert_eq!(envelope.aggregate_id, user_id);
        assert_eq!(envelope.source_service, "auth-service");
    }
}
