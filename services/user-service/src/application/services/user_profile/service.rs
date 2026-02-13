use std::sync::Arc;
use uuid::Uuid;

use crate::domain::user_profile::{UserProfile, UserProfileRepository, Username, UserStatus};

use super::error::UserProfileServiceError;

/// User Profile Application Service
///
/// Orchestrates user profile operations and enforces business rules.
pub struct UserProfileService<R>
where
    R: UserProfileRepository,
{
    repository: Arc<R>,
}

impl<R> UserProfileService<R>
where
    R: UserProfileRepository,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    // ============================================
    // PROFILE MANAGEMENT
    // ============================================

    /// Create new user profile
    pub async fn create_profile(
        &self,
        user_id: Uuid,
        username: String,
        display_name: String,
    ) -> Result<UserProfile, UserProfileServiceError> {
        // Validate username format
        let username = Username::new(&username)
            .map_err(|e| UserProfileServiceError::InvalidUsername(e.to_string()))?;

        // Check username availability
        if self.repository.exists_by_username(&username).await? {
            return Err(UserProfileServiceError::UsernameAlreadyTaken);
        }

        // Create profile
        let profile = UserProfile::new(user_id, username, display_name)
            .map_err(UserProfileServiceError::DomainError)?;

        // Save to database
        self.repository
            .save(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Get user profile by ID
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, UserProfileServiceError> {
        self.repository
            .find_by_id(user_id)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
            .ok_or(UserProfileServiceError::ProfileNotFound)
    }

    /// Get user profile by username
    pub async fn get_profile_by_username(
        &self,
        username: String,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let username = Username::new(&username)
            .map_err(|e| UserProfileServiceError::InvalidUsername(e.to_string()))?;

        self.repository
            .find_by_username(&username)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
            .ok_or(UserProfileServiceError::ProfileNotFound)
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        banner_url: Option<String>,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let mut profile = self.get_profile(user_id).await?;

        profile
            .update_profile(display_name, bio, avatar_url, banner_url)
            .map_err(UserProfileServiceError::DomainError)?;

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Change username
    pub async fn change_username(
        &self,
        user_id: Uuid,
        new_username: String,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let new_username = Username::new(&new_username)
            .map_err(|e| UserProfileServiceError::InvalidUsername(e.to_string()))?;

        if self.repository.exists_by_username(&new_username).await? {
            return Err(UserProfileServiceError::UsernameAlreadyTaken);
        }

        let mut profile = self.get_profile(user_id).await?;

        profile
            .change_username(new_username)
            .map_err(UserProfileServiceError::DomainError)?;

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Delete user profile (soft delete)
    pub async fn delete_profile(&self, user_id: Uuid) -> Result<(), UserProfileServiceError> {
        let mut profile = self.get_profile(user_id).await?;

        profile.delete();

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    // ============================================
    // PRESENCE MANAGEMENT
    // ============================================

    /// Update user status
    pub async fn update_status(
        &self,
        user_id: Uuid,
        status: UserStatus,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let mut profile = self.get_profile(user_id).await?;

        profile.update_status(status);

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Set custom status
    pub async fn set_custom_status(
        &self,
        user_id: Uuid,
        text: Option<String>,
        emoji: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let mut profile = self.get_profile(user_id).await?;

        profile
            .set_custom_status(text, emoji, expires_at)
            .map_err(UserProfileServiceError::DomainError)?;

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Clear custom status
    pub async fn clear_custom_status(
        &self,
        user_id: Uuid,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let mut profile = self.get_profile(user_id).await?;

        profile.clear_custom_status();

        self.repository
            .update(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    // ============================================
    // SEARCH & DISCOVERY
    // ============================================

    /// Search users by query
    pub async fn search_users(
        &self,
        query: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        self.repository
            .search(&query, limit, offset)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))
    }

    /// Get multiple profiles by IDs (batch)
    pub async fn get_profiles_by_ids(
        &self,
        user_ids: Vec<Uuid>,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        self.repository
            .find_by_ids(user_ids)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))
    }

    /// Get online users
    pub async fn get_online_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        self.repository
            .find_online_users(limit, offset)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))
    }

    /// Check username availability
    pub async fn is_username_available(
        &self,
        username: String,
    ) -> Result<bool, UserProfileServiceError> {
        let username = Username::new(&username)
            .map_err(|e| UserProfileServiceError::InvalidUsername(e.to_string()))?;

        self.repository
            .is_username_available(&username)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))
    }
}
