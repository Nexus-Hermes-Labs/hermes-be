use chrono::{DateTime, Utc};
use common::domain::event::IntoEventEnvelope;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event published when user_profile changes their password
///
/// This event is published when a user_profile successfully changes their password.
/// Other services can subscribe to this event to:
/// - Send security alert email (notification-service)
/// - Revoke all sessions if requested (session-service)
/// - Log security event (audit-service)
/// - Track password changes (analytics-service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPasswordChangedEvent {
    /// User ID
    pub user_id: Uuid,

    /// When the password was changed
    pub changed_at: DateTime<Utc>,

    /// If true, all sessions should be revoked (logout all devices)
    ///
    /// This is typically true when:
    /// - User suspects account compromise
    /// - User explicitly requests to logout all devices
    /// - Password reset flow
    pub revoke_all_sessions: bool,

    /// How the password was changed
    pub change_method: PasswordChangeMethod,
}

/// How the password was changed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasswordChangeMethod {
    /// User changed password while logged in
    UserInitiated,

    /// User used password reset flow (forgot password)
    PasswordReset,

    /// Admin forced password change
    AdminForced,
}

impl UserPasswordChangedEvent {
    /// Create event for user_profile-initiated password change
    pub fn new_user_initiated(user_id: Uuid, revoke_all_sessions: bool) -> Self {
        Self {
            user_id,
            changed_at: Utc::now(),
            revoke_all_sessions,
            change_method: PasswordChangeMethod::UserInitiated,
        }
    }

    /// Create event for password reset flow
    pub fn new_password_reset(user_id: Uuid) -> Self {
        Self {
            user_id,
            changed_at: Utc::now(),
            revoke_all_sessions: true, // Always revoke on reset
            change_method: PasswordChangeMethod::PasswordReset,
        }
    }

    /// Create event for admin-forced change
    pub fn new_admin_forced(user_id: Uuid) -> Self {
        Self {
            user_id,
            changed_at: Utc::now(),
            revoke_all_sessions: true, // Always revoke on admin force
            change_method: PasswordChangeMethod::AdminForced,
        }
    }

    /// Create event with custom timestamp (for testing)
    #[cfg(test)]
    pub fn new_with_timestamp(
        user_id: Uuid,
        changed_at: DateTime<Utc>,
        revoke_all_sessions: bool,
        change_method: PasswordChangeMethod,
    ) -> Self {
        Self {
            user_id,
            changed_at,
            revoke_all_sessions,
            change_method,
        }
    }
}

impl IntoEventEnvelope for UserPasswordChangedEvent {
    fn event_type(&self) -> &'static str {
        "user_profile.password.changed"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_initiated_password_change() {
        let user_id = Uuid::new_v4();
        let event = UserPasswordChangedEvent::new_user_initiated(user_id, false);

        assert_eq!(event.user_id, user_id);
        assert!(!event.revoke_all_sessions);
        assert_eq!(event.change_method, PasswordChangeMethod::UserInitiated);
        assert_eq!(event.event_type(), "user_profile.password.changed");
    }

    #[test]
    fn test_password_reset() {
        let user_id = Uuid::new_v4();
        let event = UserPasswordChangedEvent::new_password_reset(user_id);

        assert_eq!(event.user_id, user_id);
        assert!(event.revoke_all_sessions); // Always true for reset
        assert_eq!(event.change_method, PasswordChangeMethod::PasswordReset);
    }

    #[test]
    fn test_admin_forced() {
        let user_id = Uuid::new_v4();
        let event = UserPasswordChangedEvent::new_admin_forced(user_id);

        assert_eq!(event.user_id, user_id);
        assert!(event.revoke_all_sessions); // Always true for admin
        assert_eq!(event.change_method, PasswordChangeMethod::AdminForced);
    }

    #[test]
    fn test_into_envelope() {
        let user_id = Uuid::new_v4();
        let event = UserPasswordChangedEvent::new_user_initiated(user_id, true);
        let envelope = event.into_envelope("auth-service");

        assert_eq!(envelope.event_type, "user_profile.password.changed");
        assert_eq!(envelope.aggregate_id, user_id);
        assert_eq!(envelope.source_service, "auth-service");
    }
}
