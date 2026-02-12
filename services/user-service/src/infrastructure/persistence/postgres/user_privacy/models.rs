use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::user_privacy::{
    ContentFilterLevel, DmPrivacy, FriendRequestPrivacy, UserPrivacyError, UserPrivacySettings,
};

/// Database model for user_privacy_settings table
#[derive(Debug, Clone, FromRow)]
pub struct UserPrivacySettingsRow {
    pub user_id: Uuid,
    pub allow_dms_from: String,
    pub allow_friend_requests_from: String,
    pub show_online_status: bool,
    pub show_current_activity: bool,
    pub show_profile_to_non_friends: bool,
    pub allow_nsfw_content: bool,
    pub content_filter_level: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserPrivacySettingsRow> for UserPrivacySettings {
    type Error = UserPrivacyError;

    fn try_from(row: UserPrivacySettingsRow) -> Result<Self, Self::Error> {
        let allow_dms_from = row
            .allow_dms_from
            .parse::<DmPrivacy>()
            .map_err(|e| UserPrivacyError::InvalidDmPrivacy(e))?;

        let allow_friend_requests_from = row
            .allow_friend_requests_from
            .parse::<FriendRequestPrivacy>()
            .map_err(|e| UserPrivacyError::InvalidFriendRequestPrivacy(e))?;

        let content_filter_level = ContentFilterLevel::from_i16(row.content_filter_level)
            .map_err(|_| UserPrivacyError::InvalidContentFilterLevel(row.content_filter_level))?;

        Ok(UserPrivacySettings::from_persisted(
            row.user_id,
            allow_dms_from,
            allow_friend_requests_from,
            row.show_online_status,
            row.show_current_activity,
            row.show_profile_to_non_friends,
            row.allow_nsfw_content,
            content_filter_level,
            row.created_at,
            row.updated_at,
        ))
    }
}

impl From<&UserPrivacySettings> for UserPrivacySettingsRow {
    fn from(settings: &UserPrivacySettings) -> Self {
        Self {
            user_id: settings.user_id(),
            allow_dms_from: settings.allow_dms_from().as_str().to_string(),
            allow_friend_requests_from: settings
                .allow_friend_requests_from()
                .as_str()
                .to_string(),
            show_online_status: settings.show_online_status(),
            show_current_activity: settings.show_current_activity(),
            show_profile_to_non_friends: settings.show_profile_to_non_friends(),
            allow_nsfw_content: settings.allow_nsfw_content(),
            content_filter_level: settings.content_filter_level().as_i16(),
            created_at: settings.created_at(),
            updated_at: settings.updated_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_to_entity_conversion() {
        let row = UserPrivacySettingsRow {
            user_id: Uuid::new_v4(),
            allow_dms_from: "friends".to_string(),
            allow_friend_requests_from: "everyone".to_string(),
            show_online_status: true,
            show_current_activity: true,
            show_profile_to_non_friends: true,
            allow_nsfw_content: false,
            content_filter_level: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let settings = UserPrivacySettings::try_from(row).unwrap();
        assert_eq!(settings.allow_dms_from(), DmPrivacy::Friends);
        assert!(settings.show_online_status());
    }

    #[test]
    fn test_entity_to_row_conversion() {
        let settings = UserPrivacySettings::new(Uuid::new_v4());
        let row = UserPrivacySettingsRow::from(&settings);

        assert_eq!(row.allow_dms_from, "friends");
        assert_eq!(row.allow_friend_requests_from, "everyone");
        assert_eq!(row.content_filter_level, 1);
    }

    #[test]
    fn test_invalid_dm_privacy() {
        let row = UserPrivacySettingsRow {
            user_id: Uuid::new_v4(),
            allow_dms_from: "invalid".to_string(),
            allow_friend_requests_from: "everyone".to_string(),
            show_online_status: true,
            show_current_activity: true,
            show_profile_to_non_friends: true,
            allow_nsfw_content: false,
            content_filter_level: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = UserPrivacySettings::try_from(row);
        assert!(result.is_err());
    }
}
