use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// PRIVACY SETTINGS CREATED
// ============================================

/// Event published when privacy settings are created
///
/// Subject: user.privacy.created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettingsCreatedEvent {
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl PrivacySettingsCreatedEvent {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            created_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.privacy.created"
    }
}

// ============================================
// PRIVACY SETTINGS UPDATED
// ============================================

/// Event published when privacy settings are updated
///
/// Subject: user.privacy.updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettingsUpdatedEvent {
    pub user_id: Uuid,
    pub updated_settings: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl PrivacySettingsUpdatedEvent {
    pub fn new(user_id: Uuid, updated_settings: Vec<String>) -> Self {
        Self {
            user_id,
            updated_settings,
            updated_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.privacy.updated"
    }
}

// ============================================
// DM PRIVACY CHANGED
// ============================================

/// Event published when DM privacy setting changes
///
/// Subject: user.privacy.dm.changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmPrivacyChangedEvent {
    pub user_id: Uuid,
    pub old_setting: String,
    pub new_setting: String,
    pub changed_at: DateTime<Utc>,
}

impl DmPrivacyChangedEvent {
    pub fn new(user_id: Uuid, old_setting: String, new_setting: String) -> Self {
        Self {
            user_id,
            old_setting,
            new_setting,
            changed_at: Utc::now(),
        }
    }

    pub fn subject() -> &'static str {
        "user.privacy.dm.changed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_created_event() {
        let event = PrivacySettingsCreatedEvent::new(Uuid::new_v4());
        assert_eq!(
            PrivacySettingsCreatedEvent::subject(),
            "user.privacy.created"
        );
    }

    #[test]
    fn test_dm_privacy_changed_event() {
        let event = DmPrivacyChangedEvent::new(
            Uuid::new_v4(),
            "friends".to_string(),
            "none".to_string(),
        );

        assert_eq!(event.old_setting, "friends");
        assert_eq!(event.new_setting, "none");
    }
}
