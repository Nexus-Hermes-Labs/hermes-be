use crate::domain::user::error::UserDomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// =====================================================
// UserStatus — Pure Value Object (Enumeration)
// Owned by Presence Service, read-only here.
// =====================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStatus {
    Online,
    Offline,
    Idle,
    Dnd,
}

impl UserStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Idle => "idle",
            Self::Dnd => "dnd",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, UserDomainError> {
        match s {
            "online" => Ok(Self::Online),
            "offline" => Ok(Self::Offline),
            "idle" => Ok(Self::Idle),
            "dnd" => Ok(Self::Dnd),
            _ => Err(UserDomainError::InvalidUserStatus),
        }
    }
}

// =====================================================
// UserRole — Pure Value Object (Enumeration)
// Owned by Auth Service, read-only here.
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    User,
    Moderator,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::User => "user",
            Self::Moderator => "moderator",
            Self::Admin => "admin",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, UserDomainError> {
        match s {
            "user" => Ok(Self::User),
            "moderator" => Ok(Self::Moderator),
            "admin" => Ok(Self::Admin),
            _ => Err(UserDomainError::InvalidUserRole),
        }
    }

    /// Display badge text — None for regular users
    pub fn badge(&self) -> Option<&str> {
        match self {
            Self::User => None,
            Self::Moderator => Some("MOD"),
            Self::Admin => Some("ADMIN"),
        }
    }
}

// =====================================================
// CustomStatus — Value Object
// Self-validating, immutable after construction.
// =====================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomStatus {
    pub text: Option<String>,
    pub emoji: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CustomStatus {
    /// Validated constructor — enforces all domain rules
    pub fn new(
        text: Option<String>,
        emoji: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, UserDomainError> {
        if let Some(t) = &text {
            if t.trim().is_empty() {
                return Err(UserDomainError::CustomStatusTextEmpty);
            }
            if t.len() > 128 {
                return Err(UserDomainError::CustomStatusTextTooLong);
            }
        }

        if let Some(e) = &emoji {
            if e.len() > 50 {
                return Err(UserDomainError::CustomStatusEmojiTooLong);
            }
        }

        if let Some(exp) = expires_at {
            if exp <= Utc::now() {
                return Err(UserDomainError::CustomStatusExpirationInPast);
            }
        }

        Ok(Self {
            text,
            emoji,
            expires_at,
        })
    }

    /// Has this status passed its expiration time?
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Utc::now())
    }

    /// Formatted display: "🎮 Playing Rust"
    pub fn display(&self) -> String {
        match (&self.emoji, &self.text) {
            (Some(emoji), Some(text)) => format!("{} {}", emoji, text),
            (Some(emoji), None) => emoji.clone(),
            (None, Some(text)) => text.clone(),
            (None, None) => String::new(),
        }
    }
}

// =====================================================
// Privacy Value Objects
// =====================================================

/// Composite value object — groups all privacy settings together
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPrivacySettings {
    pub allow_dms_from: DmPrivacy,
    pub allow_friend_requests_from: FriendRequestPrivacy,
    pub show_online_status: bool,
}

impl Default for UserPrivacySettings {
    fn default() -> Self {
        Self {
            allow_dms_from: DmPrivacy::Friends,
            allow_friend_requests_from: FriendRequestPrivacy::Everyone,
            show_online_status: true,
        }
    }
}

/// Who can send DMs to this user?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmPrivacy {
    Everyone,
    Friends,
    ServerMembers,
    /// Nobody can send DMs (maps to "none" in DB)
    NoOne,
}

impl DmPrivacy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Everyone => "everyone",
            Self::Friends => "friends",
            Self::ServerMembers => "server_members",
            Self::NoOne => "none",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, UserDomainError> {
        match s {
            "everyone" => Ok(Self::Everyone),
            "friends" => Ok(Self::Friends),
            "server_members" => Ok(Self::ServerMembers),
            "none" => Ok(Self::NoOne),
            _ => Err(UserDomainError::InvalidDmPrivacy),
        }
    }
}

/// Who can send friend requests to this user?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendRequestPrivacy {
    Everyone,
    FriendsOfFriends,
    /// Nobody can send requests (maps to "none" in DB)
    NoOne,
}

impl FriendRequestPrivacy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Everyone => "everyone",
            Self::FriendsOfFriends => "friends_of_friends",
            Self::NoOne => "none",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, UserDomainError> {
        match s {
            "everyone" => Ok(Self::Everyone),
            "friends_of_friends" => Ok(Self::FriendsOfFriends),
            "none" => Ok(Self::NoOne),
            _ => Err(UserDomainError::InvalidFriendRequestPrivacy),
        }
    }
}

// =====================================================
// UserSnapshot — Pragmatic Value Object
// Contains an entity reference (id) + read-only display data.
// Used for cross-aggregate enrichment (friend lists, etc.)
// =====================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSnapshot {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
}

impl UserSnapshot {
    /// Build snapshot from a full User aggregate
    pub fn from_user(user: &super::entity::User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            role: user.role.clone(),
            is_active: user.is_active,
        }
    }

    /// "username#discriminator"
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username, self.discriminator)
    }

    /// Role badge for UI display
    pub fn role_badge(&self) -> Option<&str> {
        self.role.badge()
    }
}

// =====================================================
// TESTS
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::error::UserDomainError;
    use chrono::Duration;
    // ─── UserStatus ──────────────────────────────────

    #[test]
    fn test_user_status_round_trip() {
        let statuses = ["online", "offline", "idle", "dnd"];
        for s in statuses {
            let parsed = UserStatus::from_str(s).unwrap();
            assert_eq!(parsed.as_str(), s);
        }
    }

    #[test]
    fn test_user_status_invalid() {
        assert!(UserStatus::from_str("unknown").is_err());
    }

    // ─── UserRole ────────────────────────────────────

    #[test]
    fn test_user_role_badges() {
        assert!(UserRole::User.badge().is_none());
        assert_eq!(UserRole::Moderator.badge(), Some("MOD"));
        assert_eq!(UserRole::Admin.badge(), Some("ADMIN"));
    }

    #[test]
    fn test_user_role_round_trip() {
        let roles = ["user", "moderator", "admin"];
        for r in roles {
            let parsed = UserRole::from_str(r).unwrap();
            assert_eq!(parsed.as_str(), r);
        }
    }

    // ─── CustomStatus ────────────────────────────────

    #[test]
    fn test_custom_status_display_emoji_and_text() {
        let status = CustomStatus::new(
            Some("Playing Rust".to_string()),
            Some("🎮".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(status.display(), "🎮 Playing Rust");
    }

    #[test]
    fn test_custom_status_display_emoji_only() {
        let status = CustomStatus::new(None, Some("🎮".to_string()), None).unwrap();
        assert_eq!(status.display(), "🎮");
    }

    #[test]
    fn test_custom_status_display_text_only() {
        let status = CustomStatus::new(Some("AFK".to_string()), None, None).unwrap();
        assert_eq!(status.display(), "AFK");
    }

    #[test]
    fn test_custom_status_text_empty_rejected() {
        let result = CustomStatus::new(Some("   ".to_string()), None, None);
        assert!(matches!(
            result,
            Err(UserDomainError::CustomStatusTextEmpty)
        ));
    }

    #[test]
    fn test_custom_status_text_too_long() {
        let result = CustomStatus::new(Some("a".repeat(129)), None, None);
        assert!(matches!(
            result,
            Err(UserDomainError::CustomStatusTextTooLong)
        ));
    }

    #[test]
    fn test_custom_status_emoji_too_long() {
        let result = CustomStatus::new(None, Some("x".repeat(51)), None);
        assert!(matches!(
            result,
            Err(UserDomainError::CustomStatusEmojiTooLong)
        ));
    }

    #[test]
    fn test_custom_status_expiration_in_past_rejected() {
        let past = Utc::now() - Duration::hours(1);
        let result = CustomStatus::new(Some("test".to_string()), None, Some(past));
        assert!(matches!(
            result,
            Err(UserDomainError::CustomStatusExpirationInPast)
        ));
    }

    #[test]
    fn test_custom_status_not_expired_when_no_expiry() {
        let status = CustomStatus::new(Some("no expiry".to_string()), None, None).unwrap();
        assert!(!status.is_expired());
    }

    #[test]
    fn test_custom_status_expired() {
        // Bypass constructor to create an already-expired status
        let status = CustomStatus {
            text: Some("expired".to_string()),
            emoji: None,
            expires_at: Some(Utc::now() - Duration::hours(1)),
        };
        assert!(status.is_expired());
    }

    // ─── Privacy ─────────────────────────────────────

    #[test]
    fn test_dm_privacy_round_trip() {
        let values = ["everyone", "friends", "server_members", "none"];
        for v in values {
            let parsed = DmPrivacy::from_str(v).unwrap();
            assert_eq!(parsed.as_str(), v);
        }
    }

    #[test]
    fn test_friend_request_privacy_round_trip() {
        let values = ["everyone", "friends_of_friends", "none"];
        for v in values {
            let parsed = FriendRequestPrivacy::from_str(v).unwrap();
            assert_eq!(parsed.as_str(), v);
        }
    }

    #[test]
    fn test_privacy_settings_default() {
        let settings = UserPrivacySettings::default();
        assert_eq!(settings.allow_dms_from, DmPrivacy::Friends);
        assert_eq!(
            settings.allow_friend_requests_from,
            FriendRequestPrivacy::Everyone
        );
        assert!(settings.show_online_status);
    }

    // ─── UserSnapshot ────────────────────────────────

    #[test]
    fn test_user_snapshot_tag() {
        let snapshot = UserSnapshot {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            discriminator: "1234".to_string(),
            display_name: "Alice".to_string(),
            avatar_url: None,
            role: UserRole::Moderator,
            is_active: true,
        };

        assert_eq!(snapshot.tag(), "alice#1234");
        assert_eq!(snapshot.role_badge(), Some("MOD"));
    }
}
