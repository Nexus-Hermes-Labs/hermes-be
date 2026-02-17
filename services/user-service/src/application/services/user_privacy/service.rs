use std::sync::Arc;
use uuid::Uuid;

use crate::domain::user_privacy::{
    ContentFilterLevel, DmPrivacy, FriendRequestPrivacy, PrivacyPreset, Relationship,
    UserPrivacyRepository, UserPrivacySettings,
};

use super::error::UserPrivacyServiceError;

/// User Privacy Application Service
///
/// Orchestrates privacy settings operations and authorization checks.
pub struct UserPrivacyService {
    repository: Arc<dyn UserPrivacyRepository>,
}

impl UserPrivacyService {
    pub fn new(repository: Arc<dyn UserPrivacyRepository>) -> Self {
        Self { repository }
    }

    // ============================================
    // PRIVACY SETTINGS MANAGEMENT
    // ============================================

    /// Create default privacy settings for user
    pub async fn create_privacy_settings(
        &self,
        user_id: Uuid,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let settings = UserPrivacySettings::new(user_id);

        self.repository
            .save(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    /// Get privacy settings for user
    pub async fn get_privacy_settings(
        &self,
        user_id: Uuid,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        self.repository
            .find_by_id(user_id)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?
            .ok_or(UserPrivacyServiceError::PrivacySettingsNotFound { user_id })
    }

    /// Update DM privacy
    pub async fn update_dm_privacy(
        &self,
        user_id: Uuid,
        privacy: DmPrivacy,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let mut settings = self.get_privacy_settings(user_id).await?;

        settings.update_dm_privacy(privacy);

        self.repository
            .update(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    /// Update friend request privacy
    pub async fn update_friend_request_privacy(
        &self,
        user_id: Uuid,
        privacy: FriendRequestPrivacy,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let mut settings = self.get_privacy_settings(user_id).await?;

        settings.update_friend_request_privacy(privacy);

        self.repository
            .update(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    /// Update visibility settings
    pub async fn update_visibility(
        &self,
        user_id: Uuid,
        show_online_status: Option<bool>,
        show_current_activity: Option<bool>,
        show_profile_to_non_friends: Option<bool>,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let mut settings = self.get_privacy_settings(user_id).await?;

        settings.update_visibility(
            show_online_status,
            show_current_activity,
            show_profile_to_non_friends,
        );

        self.repository
            .update(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    /// Update content settings
    pub async fn update_content_settings(
        &self,
        user_id: Uuid,
        allow_nsfw: Option<bool>,
        filter_level: Option<ContentFilterLevel>,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let mut settings = self.get_privacy_settings(user_id).await?;

        settings.update_content_settings(allow_nsfw, filter_level);

        self.repository
            .update(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    /// Apply privacy preset
    pub async fn apply_preset(
        &self,
        user_id: Uuid,
        preset: PrivacyPreset,
    ) -> Result<UserPrivacySettings, UserPrivacyServiceError> {
        let mut settings = self.get_privacy_settings(user_id).await?;

        settings.apply_preset(preset);

        self.repository
            .update(&settings)
            .await
            .map_err(|e| UserPrivacyServiceError::RepositoryError(e.to_string()))?;

        Ok(settings)
    }

    // ============================================
    // AUTHORIZATION CHECKS
    // ============================================

    /// Check if user can receive DM from sender
    pub async fn can_receive_dm(
        &self,
        target_user_id: Uuid,
        relationship: Relationship,
    ) -> Result<bool, UserPrivacyServiceError> {
        let settings = self.get_privacy_settings(target_user_id).await?;
        Ok(settings.can_receive_dm_from(relationship))
    }

    /// Check if user can receive friend request from sender
    pub async fn can_receive_friend_request(
        &self,
        target_user_id: Uuid,
        relationship: Relationship,
    ) -> Result<bool, UserPrivacyServiceError> {
        let settings = self.get_privacy_settings(target_user_id).await?;
        Ok(settings.can_receive_friend_request_from(relationship))
    }

    /// Check if profile is visible to viewer
    pub async fn is_profile_visible(
        &self,
        target_user_id: Uuid,
        relationship: Relationship,
    ) -> Result<bool, UserPrivacyServiceError> {
        let settings = self.get_privacy_settings(target_user_id).await?;
        Ok(settings.is_profile_visible_to(relationship))
    }

    /// Authorize DM (throws error if not allowed)
    pub async fn authorize_dm(
        &self,
        target_user_id: Uuid,
        relationship: Relationship,
    ) -> Result<(), UserPrivacyServiceError> {
        if !self.can_receive_dm(target_user_id, relationship).await? {
            return Err(UserPrivacyServiceError::DmNotAllowed);
        }
        Ok(())
    }

    /// Authorize friend request (throws error if not allowed)
    pub async fn authorize_friend_request(
        &self,
        target_user_id: Uuid,
        relationship: Relationship,
    ) -> Result<(), UserPrivacyServiceError> {
        if !self
            .can_receive_friend_request(target_user_id, relationship)
            .await?
        {
            return Err(UserPrivacyServiceError::FriendRequestNotAllowed);
        }
        Ok(())
    }
}
