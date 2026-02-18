use std::sync::Arc;
use uuid::Uuid;

use crate::domain::user_profile::{UserProfile, UserProfileRepository, Username, UserStatus};
use crate::domain::user_relationship::UserRelationshipRepository;

use super::error::UserProfileServiceError;

/// User Profile Application Service
///
/// Orchestrates user profile operations and enforces business rules.
pub struct UserProfileService {
    repository: Arc<dyn UserProfileRepository>,
    relationship_repository: Arc<dyn UserRelationshipRepository>,
}

impl UserProfileService {
    pub fn new(
        repository: Arc<dyn UserProfileRepository>,
        relationship_repository: Arc<dyn UserRelationshipRepository>,
    ) -> Self {
        Self {
            repository,
            relationship_repository,
        }
    }

    // ============================================
    // PROFILE MANAGEMENT
    // ============================================

    /// Create new user profile
    pub async fn create_profile(
        &self,
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
        let profile = UserProfile::new(username, display_name)
            .map_err(UserProfileServiceError::DomainError)?;

        // Save to database
        self.repository
            .save(&profile)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        Ok(profile)
    }

    /// Get user profile by ID
    pub async fn get_profile(
        &self,
        user_id: Uuid,
        requester_id: Option<Uuid>,
    ) -> Result<UserProfile, UserProfileServiceError> {
        // Check if the target user has blocked the requester
        if let Some(req_id) = requester_id {
            if let Some(rel) = self
                .relationship_repository
                .find_by_user_and_target(user_id, req_id)
                .await
                .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
            {
                if rel.is_blocked() {
                    return Err(UserProfileServiceError::ProfileNotFound);
                }
            }
        }

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
        requester_id: Option<Uuid>,
    ) -> Result<UserProfile, UserProfileServiceError> {
        let username_vo = Username::new(&username)
            .map_err(|e| UserProfileServiceError::InvalidUsername(e.to_string()))?;

        let profile = self
            .repository
            .find_by_username(&username_vo)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
            .ok_or(UserProfileServiceError::ProfileNotFound)?;

        // Check if the target user has blocked the requester
        if let Some(req_id) = requester_id {
            if let Some(rel) = self
                .relationship_repository
                .find_by_user_and_target(profile.id(), req_id)
                .await
                .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
            {
                if rel.is_blocked() {
                    return Err(UserProfileServiceError::ProfileNotFound);
                }
            }
        }

        Ok(profile)
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
        let mut profile = self.get_profile(user_id, None).await?;

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

        let mut profile = self.get_profile(user_id, None).await?;

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
        let mut profile = self.get_profile(user_id, None).await?;

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
        let mut profile = self.get_profile(user_id, None).await?;

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
        let mut profile = self.get_profile(user_id, None).await?;

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
        let mut profile = self.get_profile(user_id, None).await?;

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
        requester_id: Option<Uuid>,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        let profiles = self
            .repository
            .search(&query, limit, offset)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        if let Some(req_id) = requester_id {
            let mut filtered = Vec::new();
            for profile in profiles {
                if !self
                    .relationship_repository
                    .is_blocked_bidirectional(req_id, profile.id())
                    .await
                    .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
                {
                    filtered.push(profile);
                }
            }
            Ok(filtered)
        } else {
            Ok(profiles)
        }
    }

    /// Get multiple profiles by IDs (batch)
    pub async fn get_profiles_by_ids(
        &self,
        user_ids: Vec<Uuid>,
        requester_id: Option<Uuid>,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        let profiles = self
            .repository
            .find_by_ids(user_ids)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        if let Some(req_id) = requester_id {
            let mut filtered = Vec::new();
            for profile in profiles {
                if !self
                    .relationship_repository
                    .is_blocked_bidirectional(req_id, profile.id())
                    .await
                    .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
                {
                    filtered.push(profile);
                }
            }
            Ok(filtered)
        } else {
            Ok(profiles)
        }
    }

    /// Get online users
    pub async fn get_online_users(
        &self,
        limit: i64,
        offset: i64,
        requester_id: Option<Uuid>,
    ) -> Result<Vec<UserProfile>, UserProfileServiceError> {
        let profiles = self
            .repository
            .find_online_users(limit, offset)
            .await
            .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?;

        if let Some(req_id) = requester_id {
            let mut filtered = Vec::new();
            for profile in profiles {
                if !self
                    .relationship_repository
                    .is_blocked_bidirectional(req_id, profile.id())
                    .await
                    .map_err(|e| UserProfileServiceError::RepositoryError(e.to_string()))?
                {
                    filtered.push(profile);
                }
            }
            Ok(filtered)
        } else {
            Ok(profiles)
        }
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
