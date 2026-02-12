use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::user_profile::{UserProfile, UserProfileError, Username, UserStatus};

/// Database model for user_profiles table
#[derive(Debug, Clone, FromRow)]
pub struct UserProfileRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub bio: Option<String>,
    pub status: String,
    pub custom_status_text: Option<String>,
    pub custom_status_emoji: Option<String>,
    pub custom_status_expires_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub last_username_changed_at: Option<DateTime<Utc>>,
}

impl TryFrom<UserProfileRow> for UserProfile {
    type Error = UserProfileError;

    fn try_from(row: UserProfileRow) -> Result<Self, Self::Error> {
        let username = Username::new(&row.username)?;
        let status = row.status.parse::<UserStatus>()?;

        Ok(UserProfile::from_persisted(
            row.id,
            username,
            row.display_name,
            row.avatar_url,
            row.banner_url,
            row.bio,
            status,
            row.custom_status_text,
            row.custom_status_emoji,
            row.custom_status_expires_at,
            row.last_seen_at,
            row.created_at,
            row.updated_at,
            row.deleted_at,
            row.last_username_changed_at,
        ))
    }
}

impl From<&UserProfile> for UserProfileRow {
    fn from(profile: &UserProfile) -> Self {
        Self {
            id: profile.id(),
            username: profile.username().as_str().to_string(),
            display_name: profile.display_name().to_string(),
            avatar_url: profile.avatar_url().map(String::from),
            banner_url: profile.banner_url().map(String::from),
            bio: profile.bio().map(String::from),
            status: profile.status().as_str().to_string(),
            custom_status_text: profile.custom_status_text().map(String::from),
            custom_status_emoji: profile.custom_status_emoji().map(String::from),
            custom_status_expires_at: profile.custom_status_expires_at(),
            last_seen_at: profile.last_seen_at(),
            created_at: profile.created_at(),
            updated_at: profile.updated_at(),
            deleted_at: if profile.is_deleted() {
                Some(profile.updated_at())
            } else {
                None
            },
            last_username_changed_at: None, // Populated from entity if available
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_to_entity_conversion() {
        let row = UserProfileRow {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            banner_url: None,
            bio: None,
            status: "online".to_string(),
            custom_status_text: None,
            custom_status_emoji: None,
            custom_status_expires_at: None,
            last_seen_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            last_username_changed_at: None,
        };

        let profile = UserProfile::try_from(row).unwrap();
        assert_eq!(profile.username().as_str(), "testuser");
        assert_eq!(profile.display_name(), "Test User");
    }

    #[test]
    fn test_entity_to_row_conversion() {
        let username = Username::new("testuser").unwrap();
        let profile = UserProfile::new(
            Uuid::new_v4(),
            username,
            "Test User".to_string(),
        )
        .unwrap();

        let row = UserProfileRow::from(&profile);
        assert_eq!(row.username, "testuser");
        assert_eq!(row.display_name, "Test User");
    }
}
