use crate::domain::user::error::UserDomainError;
use crate::domain::user::valueobject::{
    CustomStatus, DmPrivacy, FriendRequestPrivacy, UserPrivacySettings, UserStatus,
};
use crate::domain::user::UserRole;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// User aggregate root — User Service perspective.
///
/// Domain ownership map:
///   Auth Service → role, is_active (read-only here)
///   User Service → profile, privacy (owned and modified here)
///   Presence Svc → status, custom_status (read-only here)
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,

    // ─── Identity ────────────────────────────────────
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    // ─── Profile ─────────────────────────────────────
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub bio: Option<String>,

    // ─── Status (Presence Service owns) ──────────────
    pub status: UserStatus,
    pub custom_status: Option<CustomStatus>,

    // ─── Privacy (User Service owns) ─────────────────
    pub privacy_settings: UserPrivacySettings,

    // ─── Read-only (Auth Service owns) ───────────────
    pub role: UserRole,
    pub is_active: bool,

    // ─── Metadata ────────────────────────────────────
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =====================================================
// DOMAIN BEHAVIORS
// =====================================================

impl User {
    /// Update user profile fields.
    /// Only touches fields owned by User Service.
    pub fn update_profile(
        &mut self,
        display_name: Option<String>,
        avatar_url: Option<String>,
        banner_url: Option<String>,
        bio: Option<String>,
    ) -> Result<(), UserDomainError> {
        if let Some(name) = display_name {
            if name.trim().is_empty() {
                return Err(UserDomainError::InvalidDisplayName);
            }
            if name.len() > 100 {
                return Err(UserDomainError::DisplayNameTooLong);
            }
            self.display_name = name.trim().to_string();
        }

        if let Some(url) = avatar_url {
            Self::validate_url(&url)?;
            self.avatar_url = Some(url);
        }

        if let Some(url) = banner_url {
            Self::validate_url(&url)?;
            self.banner_url = Some(url);
        }

        if let Some(b) = bio {
            if b.len() > 500 {
                return Err(UserDomainError::BioTooLong);
            }
            self.bio = if b.trim().is_empty() { None } else { Some(b) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set or replace custom status.
    /// Pass (None, None, _) to clear.
    pub fn set_custom_status(
        &mut self,
        text: Option<String>,
        emoji: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), UserDomainError> {
        self.custom_status = match (&text, &emoji) {
            (None, None) => None,
            _ => Some(CustomStatus::new(text, emoji, expires_at)?),
        };
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Unconditionally clear custom status
    pub fn clear_custom_status(&mut self) {
        self.custom_status = None;
        self.updated_at = Utc::now();
    }

    /// If current custom status has expired, clear it.
    /// Returns true when status was actually cleared.
    pub fn clear_expired_custom_status(&mut self) -> bool {
        let expired = self
            .custom_status
            .as_ref()
            .map_or(false, |s| s.is_expired());

        if expired {
            self.custom_status = None;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Update online presence status
    pub fn set_status(&mut self, status: UserStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Replace all privacy settings at once
    pub fn update_privacy_settings(&mut self, settings: UserPrivacySettings) {
        self.privacy_settings = settings;
        self.updated_at = Utc::now();
    }
}

// =====================================================
// DOMAIN QUERIES (read-only, no side effects)
// =====================================================

impl User {
    /// Can this user receive a DM given the relationship context?
    pub fn can_receive_dm_from(&self, are_friends: bool, share_server: bool) -> bool {
        match self.privacy_settings.allow_dms_from {
            DmPrivacy::Everyone => true,
            DmPrivacy::Friends => are_friends,
            DmPrivacy::ServerMembers => share_server,
            DmPrivacy::NoOne => false,
        }
    }

    /// Can this user receive a friend request given the relationship context?
    pub fn can_receive_friend_request(&self, are_friends_of_friends: bool) -> bool {
        match self.privacy_settings.allow_friend_requests_from {
            FriendRequestPrivacy::Everyone => true,
            FriendRequestPrivacy::FriendsOfFriends => are_friends_of_friends,
            FriendRequestPrivacy::NoOne => false,
        }
    }

    /// Returns the status that should be visible to other users.
    /// Respects the show_online_status privacy flag.
    pub fn visible_status(&self) -> UserStatus {
        if self.privacy_settings.show_online_status {
            self.status.clone()
        } else {
            UserStatus::Offline
        }
    }

    /// Does this user have moderator-level (or higher) privileges?
    pub fn is_moderator_or_above(&self) -> bool {
        matches!(self.role, UserRole::Moderator | UserRole::Admin)
    }
}

// =====================================================
// PRIVATE HELPERS
// =====================================================

impl User {
    fn validate_url(url: &str) -> Result<(), UserDomainError> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(())
        } else {
            Err(UserDomainError::InvalidUrl)
        }
    }
}

// =====================================================
// TESTS
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::error::UserDomainError;
    use crate::domain::user::valueobject::{DmPrivacy, FriendRequestPrivacy};
    use chrono::Duration;

    /// Helper: build a minimal valid User for testing
    fn stub_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            discriminator: "0000".to_string(),
            display_name: "Alice".to_string(),
            avatar_url: None,
            banner_url: None,
            bio: None,
            status: UserStatus::Online,
            custom_status: None,
            privacy_settings: UserPrivacySettings::default(),
            role: UserRole::User,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ─── update_profile ──────────────────────────────

    #[test]
    fn test_update_display_name_ok() {
        let mut user = stub_user();
        user.update_profile(Some("New Name".into()), None, None, None)
            .unwrap();
        assert_eq!(user.display_name, "New Name");
    }

    #[test]
    fn test_update_display_name_trims_whitespace() {
        let mut user = stub_user();
        user.update_profile(Some("  Trimmed  ".into()), None, None, None)
            .unwrap();
        assert_eq!(user.display_name, "Trimmed");
    }

    #[test]
    fn test_update_display_name_empty_rejected() {
        let mut user = stub_user();
        let result = user.update_profile(Some("   ".into()), None, None, None);
        assert!(matches!(result, Err(UserDomainError::InvalidDisplayName)));
    }

    #[test]
    fn test_update_display_name_too_long() {
        let mut user = stub_user();
        let result = user.update_profile(Some("a".repeat(101)), None, None, None);
        assert!(matches!(result, Err(UserDomainError::DisplayNameTooLong)));
    }

    #[test]
    fn test_update_avatar_url_valid() {
        let mut user = stub_user();
        user.update_profile(
            None,
            Some("https://cdn.example.com/avatar.png".into()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            user.avatar_url,
            Some("https://cdn.example.com/avatar.png".into())
        );
    }

    #[test]
    fn test_update_avatar_url_invalid() {
        let mut user = stub_user();
        let result = user.update_profile(None, Some("ftp://bad.url/avatar.png".into()), None, None);
        assert!(matches!(result, Err(UserDomainError::InvalidUrl)));
    }

    #[test]
    fn test_update_banner_url_valid() {
        let mut user = stub_user();
        user.update_profile(
            None,
            None,
            Some("https://cdn.example.com/banner.png".into()),
            None,
        )
        .unwrap();
        assert_eq!(
            user.banner_url,
            Some("https://cdn.example.com/banner.png".into())
        );
    }

    #[test]
    fn test_update_bio_ok() {
        let mut user = stub_user();
        user.update_profile(None, None, None, Some("Hello world".into()))
            .unwrap();
        assert_eq!(user.bio, Some("Hello world".into()));
    }

    #[test]
    fn test_update_bio_empty_clears() {
        let mut user = stub_user();
        user.bio = Some("old bio".into());
        user.update_profile(None, None, None, Some("   ".into()))
            .unwrap();
        assert!(user.bio.is_none());
    }

    #[test]
    fn test_update_bio_too_long() {
        let mut user = stub_user();
        let result = user.update_profile(None, None, None, Some("x".repeat(501)));
        assert!(matches!(result, Err(UserDomainError::BioTooLong)));
    }

    // ─── custom_status ───────────────────────────────

    #[test]
    fn test_set_custom_status() {
        let mut user = stub_user();
        let future = Utc::now() + Duration::hours(1);
        user.set_custom_status(Some("Playing".into()), Some("🎮".into()), Some(future))
            .unwrap();

        let cs = user.custom_status.as_ref().unwrap();
        assert_eq!(cs.display(), "🎮 Playing");
        assert!(!cs.is_expired());
    }

    #[test]
    fn test_set_custom_status_none_none_clears() {
        let mut user = stub_user();
        user.custom_status = Some(CustomStatus {
            text: Some("old".into()),
            emoji: None,
            expires_at: None,
        });

        user.set_custom_status(None, None, None).unwrap();
        assert!(user.custom_status.is_none());
    }

    #[test]
    fn test_clear_custom_status() {
        let mut user = stub_user();
        user.custom_status = Some(CustomStatus {
            text: Some("test".into()),
            emoji: None,
            expires_at: None,
        });

        user.clear_custom_status();
        assert!(user.custom_status.is_none());
    }

    #[test]
    fn test_clear_expired_custom_status_returns_true() {
        let mut user = stub_user();
        user.custom_status = Some(CustomStatus {
            text: Some("expired".into()),
            emoji: None,
            expires_at: Some(Utc::now() - Duration::hours(1)),
        });

        assert!(user.clear_expired_custom_status());
        assert!(user.custom_status.is_none());
    }

    #[test]
    fn test_clear_expired_custom_status_noop_when_valid() {
        let mut user = stub_user();
        user.custom_status = Some(CustomStatus {
            text: Some("still valid".into()),
            emoji: None,
            expires_at: Some(Utc::now() + Duration::hours(1)),
        });

        assert!(!user.clear_expired_custom_status());
        assert!(user.custom_status.is_some());
    }

    // ─── privacy queries ─────────────────────────────

    #[test]
    fn test_can_receive_dm_everyone() {
        let mut user = stub_user();
        user.privacy_settings.allow_dms_from = DmPrivacy::Everyone;
        assert!(user.can_receive_dm_from(false, false));
    }

    #[test]
    fn test_can_receive_dm_friends_only() {
        let mut user = stub_user();
        user.privacy_settings.allow_dms_from = DmPrivacy::Friends;
        assert!(!user.can_receive_dm_from(false, true));
        assert!(user.can_receive_dm_from(true, false));
    }

    #[test]
    fn test_can_receive_dm_server_members() {
        let mut user = stub_user();
        user.privacy_settings.allow_dms_from = DmPrivacy::ServerMembers;
        assert!(!user.can_receive_dm_from(false, false));
        assert!(user.can_receive_dm_from(false, true));
    }

    #[test]
    fn test_can_receive_dm_no_one() {
        let mut user = stub_user();
        user.privacy_settings.allow_dms_from = DmPrivacy::NoOne;
        assert!(!user.can_receive_dm_from(true, true));
    }

    #[test]
    fn test_can_receive_friend_request_everyone() {
        let mut user = stub_user();
        user.privacy_settings.allow_friend_requests_from = FriendRequestPrivacy::Everyone;
        assert!(user.can_receive_friend_request(false));
    }

    #[test]
    fn test_can_receive_friend_request_fof_only() {
        let mut user = stub_user();
        user.privacy_settings.allow_friend_requests_from = FriendRequestPrivacy::FriendsOfFriends;
        assert!(!user.can_receive_friend_request(false));
        assert!(user.can_receive_friend_request(true));
    }

    #[test]
    fn test_can_receive_friend_request_no_one() {
        let mut user = stub_user();
        user.privacy_settings.allow_friend_requests_from = FriendRequestPrivacy::NoOne;
        assert!(!user.can_receive_friend_request(true));
    }

    // ─── visible_status ──────────────────────────────

    #[test]
    fn test_visible_status_when_show_enabled() {
        let mut user = stub_user();
        user.status = UserStatus::Dnd;
        user.privacy_settings.show_online_status = true;
        assert_eq!(user.visible_status(), UserStatus::Dnd);
    }

    #[test]
    fn test_visible_status_when_show_disabled() {
        let mut user = stub_user();
        user.status = UserStatus::Online;
        user.privacy_settings.show_online_status = false;
        assert_eq!(user.visible_status(), UserStatus::Offline);
    }

    // ─── role queries ────────────────────────────────

    #[test]
    fn test_is_moderator_or_above() {
        let mut user = stub_user();

        user.role = UserRole::User;
        assert!(!user.is_moderator_or_above());

        user.role = UserRole::Moderator;
        assert!(user.is_moderator_or_above());

        user.role = UserRole::Admin;
        assert!(user.is_moderator_or_above());
    }
}
