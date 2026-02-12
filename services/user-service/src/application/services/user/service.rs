use std::sync::Arc;
use uuid::Uuid;

use super::error::UserApplicationError;
use crate::presentation::http::dto::user::{
    CustomStatusDto, MyProfileResponse, PrivacySettingsDto, SetCustomStatusRequest,
    UpdatePrivacySettingsRequest, UpdateProfileRequest, UserProfileResponse, UserSearchResult,
    UserUpdateResponse,
};
use crate::domain::user_profile::entity::User;
use crate::domain::user_profile::repository::UserRepository;
use common::pagination::{Paginated, PaginationParams};
use common::persistence::error::RepositoryError;
use tracing::{error, info, warn};

/// User Service application orchestrator
///
/// Responsibilities:
///   - Coordinate domain entities and repositories
///   - DTO conversions (domain ↔ presentation)
///   - Transaction boundaries (future: with UnitOfWork)
///   - Logging
///
///  TODO: Separation of user_profile and privacy tables
/// No business logic — that lives in domain entities and domain services.
pub struct UserApplicationService<UR>
where
    UR: UserRepository<Error = RepositoryError>,
{
    user_repository: Arc<UR>,
}

impl<UR> UserApplicationService<UR>
where
    UR: UserRepository<Error = RepositoryError>,
{
    pub fn new(user_repository: Arc<UR>) -> Self {
        Self { user_repository }
    }

    // ─── Profile Queries ─────────────────────────────────────────────────────

    /// Get current user_profile's own profile (includes private privacy settings)
    pub async fn get_my_profile(
        &self,
        user_id: Uuid,
    ) -> Result<MyProfileResponse, UserApplicationError> {
        let user = self.get_user_by_id(user_id).await?;

        info!(user_id = %user_id, "Retrieved own profile");

        Ok(MyProfileResponse::from(&user))
    }

    /// Get another user_profile's public profile (privacy-aware status visibility)
    pub async fn get_user_profile(
        &self,
        user_id: Uuid,
        _viewer_id: Uuid, // TODO: check friendship / server membership for status visibility
    ) -> Result<UserProfileResponse, UserApplicationError> {
        let user = self.get_user_by_id(user_id).await?;

        // TODO: determine viewer_can_see_status based on:
        //   - user_profile.privacy_settings.show_online_status
        //   - are they friends?
        //   - do they share a server?
        let viewer_can_see_status = user.privacy_settings.show_online_status;

        info!(
            user_id = %user_id,
            viewer_id = %_viewer_id,
            "Retrieved user_profile profile"
        );

        Ok(UserProfileResponse::public(&user, viewer_can_see_status))
    }

    /// Get user_profile by username (for friend requests, mentions, etc.)
    pub async fn get_user_by_username(
        &self,
        username: &str,
        _viewer_id: Uuid,
    ) -> Result<UserProfileResponse, UserApplicationError> {
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| {
                warn!(username = %username, "User not found by username");
                UserApplicationError::UsernameNotFound(username.to_string())
            })?;

        // TODO: privacy-aware status
        let viewer_can_see_status = user.privacy_settings.show_online_status;

        info!(
            user_id = %user.id,
            username = %username,
            "Retrieved user_profile by username"
        );

        Ok(UserProfileResponse::public(&user, viewer_can_see_status))
    }

    // ─── Profile Management ──────────────────────────────────────────────────

    /// Update user_profile profile fields (display_name, avatar, banner, bio)
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<UserUpdateResponse, UserApplicationError> {
        if request.is_empty() {
            return Err(UserApplicationError::InvalidInput(
                "At least one field must be provided".to_string(),
            ));
        }

        let mut user = self.get_user_by_id(user_id).await?;

        // Domain entity validates and applies changes
        user.update_profile(
            request.display_name,
            request.avatar_url,
            request.banner_url,
            request.bio,
        )?;

        // Persist changes
        self.user_repository.update(&user).await?;

        info!(
            user_id = %user_id,
            "Profile updated successfully"
        );

        Ok(UserUpdateResponse::from(&user))
    }

    // ─── Privacy Settings ────────────────────────────────────────────────────

    /// Update privacy settings
    pub async fn update_privacy_settings(
        &self,
        user_id: Uuid,
        request: UpdatePrivacySettingsRequest,
    ) -> Result<PrivacySettingsDto, UserApplicationError> {
        let mut user = self.get_user_by_id(user_id).await?;

        // Convert DTO → domain value object (validates enum values)
        let settings = request.to_domain()?;

        // Domain entity applies changes
        user.update_privacy_settings(settings);

        // Persist
        self.user_repository.update(&user).await?;

        info!(
            user_id = %user_id,
            "Privacy settings updated successfully"
        );

        Ok(PrivacySettingsDto::from(&user.privacy_settings))
    }

    // ─── Custom Status ───────────────────────────────────────────────────────

    /// Set or update custom status
    pub async fn set_custom_status(
        &self,
        user_id: Uuid,
        request: SetCustomStatusRequest,
    ) -> Result<CustomStatusDto, UserApplicationError> {
        let mut user = self.get_user_by_id(user_id).await?;

        // Domain entity validates and sets status
        user.set_custom_status(request.text, request.emoji, request.expires_at)?;

        // Persist
        self.user_repository.update(&user).await?;

        info!(
            user_id = %user_id,
            "Custom status set successfully"
        );

        // Must exist after set_custom_status succeeded
        Ok(CustomStatusDto::from(user.custom_status.as_ref().unwrap()))
    }

    /// Clear custom status
    pub async fn clear_custom_status(&self, user_id: Uuid) -> Result<(), UserApplicationError> {
        let mut user = self.get_user_by_id(user_id).await?;

        user.clear_custom_status();

        self.user_repository.update(&user).await?;

        info!(user_id = %user_id, "Custom status cleared");

        Ok(())
    }

    /// Cleanup expired custom statuses (background job)
    #[allow(dead_code)]
    pub async fn cleanup_expired_statuses(&self) -> Result<(), UserApplicationError> {
        // TODO: batch query users with expired custom_status_expires_at
        // For each: user_profile.clear_expired_custom_status() and update if changed
        warn!("cleanup_expired_statuses not yet implemented");
        Ok(())
    }

    // ─── Search ──────────────────────────────────────────────────────────────

    /// Full-text search users
    pub async fn search_users(
        &self,
        query: &str,
        params: PaginationParams,
    ) -> Result<Paginated<UserSearchResult>, UserApplicationError> {
        if query.trim().is_empty() {
            return Err(UserApplicationError::InvalidInput(
                "Search query cannot be empty".to_string(),
            ));
        }

        let results = self.user_repository.search(query, &params).await?;

        let search_results = Paginated::new(
            results.items.iter().map(UserSearchResult::from).collect(),
            results.total,
            results.page,
            results.page_size,
        );

        info!(
            query = %query,
            total_results = results.total,
            "User search completed"
        );

        Ok(search_results)
    }

    // ─── Internal Helpers ────────────────────────────────────────────────────

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, UserApplicationError> {
        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "User not found");
                UserApplicationError::UserNotFound
            })
    }
}
