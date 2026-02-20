use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::services::authentication::error::AuthApplicationError;

/// Response from user profile creation
#[derive(Debug, Clone)]
pub struct UserProfileInfo {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Port for creating user profiles in user-service
///
/// This trait abstracts the communication with user-service,
/// following DDD's dependency inversion principle.
/// The application layer defines the interface; infrastructure implements it.
#[async_trait]
pub trait UserProfileClient: Send + Sync {
    async fn create_profile(
        &self,
        username: String,
        display_name: String,
        email: String,
    ) -> Result<UserProfileInfo, AuthApplicationError>;

    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileInfo, AuthApplicationError>;
}
