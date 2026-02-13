use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Response from user profile creation
#[derive(Debug, Clone)]
pub struct UserProfileInfo {
    pub user_id: Uuid,
    pub username: String,
    pub discriminator: String,
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
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a user profile in user-service
    async fn create_profile(
        &self,
        user_id: Uuid,
        username: &str,
        display_name: &str,
        email: &str,
    ) -> Result<UserProfileInfo, Self::Error>;

    /// Get a user profile from user-service
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileInfo, Self::Error>;
}
