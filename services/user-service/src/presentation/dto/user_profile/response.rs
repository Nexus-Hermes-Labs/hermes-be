use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::user_profile::UserProfile;

// ============================================
// PROFILE RESPONSE
// ============================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub bio: Option<String>,
    pub status: String,
    pub custom_status: Option<CustomStatusResponse>,
    pub created_at: DateTime<Utc>,
}

impl From<UserProfile> for ProfileResponse {
    fn from(profile: UserProfile) -> Self {
        let custom_status = if profile.custom_status_text().is_some()
            || profile.custom_status_emoji().is_some()
        {
            Some(CustomStatusResponse {
                text: profile.custom_status_text().map(String::from),
                emoji: profile.custom_status_emoji().map(String::from),
                expires_at: profile.custom_status_expires_at(),
            })
        } else {
            None
        };

        Self {
            user_id: profile.id(),
            username: profile.username().as_str().to_string(),
            display_name: profile.display_name().to_string(),
            avatar_url: profile.avatar_url().map(String::from),
            banner_url: profile.banner_url().map(String::from),
            bio: profile.bio().map(String::from),
            status: profile.status().as_str().to_string(),
            custom_status,
            created_at: profile.created_at(),
        }
    }
}

// ============================================
// CUSTOM STATUS RESPONSE
// ============================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CustomStatusResponse {
    pub text: Option<String>,
    pub emoji: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================
// PROFILE LIST RESPONSE
// ============================================

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileListResponse {
    pub profiles: Vec<ProfileResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================================
// USERNAME AVAILABILITY RESPONSE
// ============================================

#[derive(Debug, Serialize, ToSchema)]
pub struct UsernameAvailabilityResponse {
    pub username: String,
    pub available: bool,
}

// ============================================
// ONLINE USERS RESPONSE
// ============================================

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlineUsersResponse {
    pub users: Vec<ProfileResponse>,
    pub total: i64,
}
