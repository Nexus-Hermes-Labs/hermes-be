use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::valueobject::{ContentFilterLevel, DmPrivacy, FriendRequestPrivacy};

/// User Privacy Settings Aggregate Root
///
/// Separate aggregate from UserProfile to allow independent updates.
#[derive(Debug, Clone)]
pub struct UserPrivacySettings {
    // Identity
    user_id: Uuid,
    
    // DM & Friend Privacy
    allow_dms_from: DmPrivacy,
    allow_friend_requests_from: FriendRequestPrivacy,
    
    // Visibility
    show_online_status: bool,
    show_current_activity: bool,
    show_profile_to_non_friends: bool,
    
    // Content
    allow_nsfw_content: bool,
    content_filter_level: ContentFilterLevel,
    
    // Metadata
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserPrivacySettings {
    // ============================================
    // CONSTRUCTION
    // ============================================
    
    /// Create new privacy settings with defaults
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        
        Self {
            user_id,
            allow_dms_from: DmPrivacy::default(),
            allow_friend_requests_from: FriendRequestPrivacy::default(),
            show_online_status: true,
            show_current_activity: true,
            show_profile_to_non_friends: true,
            allow_nsfw_content: false,
            content_filter_level: ContentFilterLevel::default(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        user_id: Uuid,
        allow_dms_from: DmPrivacy,
        allow_friend_requests_from: FriendRequestPrivacy,
        show_online_status: bool,
        show_current_activity: bool,
        show_profile_to_non_friends: bool,
        allow_nsfw_content: bool,
        content_filter_level: ContentFilterLevel,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            allow_dms_from,
            allow_friend_requests_from,
            show_online_status,
            show_current_activity,
            show_profile_to_non_friends,
            allow_nsfw_content,
            content_filter_level,
            created_at,
            updated_at,
        }
    }
    
    // ============================================
    // GETTERS
    // ============================================
    
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    
    pub fn allow_dms_from(&self) -> DmPrivacy {
        self.allow_dms_from
    }
    
    pub fn allow_friend_requests_from(&self) -> FriendRequestPrivacy {
        self.allow_friend_requests_from
    }
    
    pub fn show_online_status(&self) -> bool {
        self.show_online_status
    }
    
    pub fn show_current_activity(&self) -> bool {
        self.show_current_activity
    }
    
    pub fn show_profile_to_non_friends(&self) -> bool {
        self.show_profile_to_non_friends
    }
    
    pub fn allow_nsfw_content(&self) -> bool {
        self.allow_nsfw_content
    }
    
    pub fn content_filter_level(&self) -> ContentFilterLevel {
        self.content_filter_level
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    // ============================================
    // BUSINESS LOGIC - Update Settings
    // ============================================
    
    /// Update DM privacy settings
    pub fn update_dm_privacy(&mut self, privacy: DmPrivacy) {
        self.allow_dms_from = privacy;
        self.updated_at = Utc::now();
    }
    
    /// Update friend request privacy settings
    pub fn update_friend_request_privacy(&mut self, privacy: FriendRequestPrivacy) {
        self.allow_friend_requests_from = privacy;
        self.updated_at = Utc::now();
    }
    
    /// Update visibility settings
    pub fn update_visibility(
        &mut self,
        show_online_status: Option<bool>,
        show_current_activity: Option<bool>,
        show_profile_to_non_friends: Option<bool>,
    ) {
        if let Some(show_online) = show_online_status {
            self.show_online_status = show_online;
        }
        
        if let Some(show_activity) = show_current_activity {
            self.show_current_activity = show_activity;
        }
        
        if let Some(show_profile) = show_profile_to_non_friends {
            self.show_profile_to_non_friends = show_profile;
        }
        
        self.updated_at = Utc::now();
    }
    
    /// Update content settings
    pub fn update_content_settings(
        &mut self,
        allow_nsfw: Option<bool>,
        filter_level: Option<ContentFilterLevel>,
    ) {
        if let Some(nsfw) = allow_nsfw {
            self.allow_nsfw_content = nsfw;
        }
        
        if let Some(level) = filter_level {
            self.content_filter_level = level;
        }
        
        self.updated_at = Utc::now();
    }
    
    /// Apply privacy preset
    pub fn apply_preset(&mut self, preset: PrivacyPreset) {
        match preset {
            PrivacyPreset::Public => {
                self.allow_dms_from = DmPrivacy::Everyone;
                self.allow_friend_requests_from = FriendRequestPrivacy::Everyone;
                self.show_online_status = true;
                self.show_current_activity = true;
                self.show_profile_to_non_friends = true;
            }
            PrivacyPreset::FriendsOnly => {
                self.allow_dms_from = DmPrivacy::Friends;
                self.allow_friend_requests_from = FriendRequestPrivacy::FriendsOfFriends;
                self.show_online_status = true;
                self.show_current_activity = true;
                self.show_profile_to_non_friends = false;
            }
            PrivacyPreset::Private => {
                self.allow_dms_from = DmPrivacy::None;
                self.allow_friend_requests_from = FriendRequestPrivacy::None;
                self.show_online_status = false;
                self.show_current_activity = false;
                self.show_profile_to_non_friends = false;
            }
        }
        
        self.updated_at = Utc::now();
    }
    
    // ============================================
    // BUSINESS LOGIC - Authorization Checks
    // ============================================
    
    /// Check if user can receive DM from given relationship
    pub fn can_receive_dm_from(&self, relationship: Relationship) -> bool {
        match self.allow_dms_from {
            DmPrivacy::Everyone => relationship != Relationship::Blocked,
            DmPrivacy::Friends => relationship == Relationship::Friend,
            DmPrivacy::ServerMembers => {
                relationship == Relationship::Friend
                    || relationship == Relationship::ServerMember
            }
            DmPrivacy::None => false,
        }
    }
    
    /// Check if user can receive friend request from given relationship
    pub fn can_receive_friend_request_from(&self, relationship: Relationship) -> bool {
        match self.allow_friend_requests_from {
            FriendRequestPrivacy::Everyone => relationship != Relationship::Blocked,
            FriendRequestPrivacy::FriendsOfFriends => {
                relationship == Relationship::FriendOfFriend
            }
            FriendRequestPrivacy::None => false,
        }
    }
    
    /// Check if profile should be visible to non-friend
    pub fn is_profile_visible_to(&self, relationship: Relationship) -> bool {
        match relationship {
            Relationship::Friend => true,
            Relationship::Blocked => false,
            _ => self.show_profile_to_non_friends,
        }
    }
}

// ============================================
// SUPPORTING TYPES
// ============================================

/// Privacy preset templates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyPreset {
    /// All features public
    Public,
    
    /// Friends only
    FriendsOnly,
    
    /// Maximum privacy
    Private,
}

/// User relationship types (for authorization)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    /// Direct friend
    Friend,
    
    /// Friend of a friend
    FriendOfFriend,
    
    /// Member of same server
    ServerMember,
    
    /// No relationship
    Stranger,
    
    /// Blocked
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_privacy_settings() {
        let user_id = Uuid::new_v4();
        let settings = UserPrivacySettings::new(user_id);
        
        assert_eq!(settings.user_id(), user_id);
        assert_eq!(settings.allow_dms_from(), DmPrivacy::Friends);
        assert!(settings.show_online_status());
    }

    #[test]
    fn test_update_dm_privacy() {
        let user_id = Uuid::new_v4();
        let mut settings = UserPrivacySettings::new(user_id);
        
        settings.update_dm_privacy(DmPrivacy::None);
        
        assert_eq!(settings.allow_dms_from(), DmPrivacy::None);
    }

    #[test]
    fn test_can_receive_dm_from() {
        let user_id = Uuid::new_v4();
        let settings = UserPrivacySettings::new(user_id);
        
        // Default is Friends
        assert!(settings.can_receive_dm_from(Relationship::Friend));
        assert!(!settings.can_receive_dm_from(Relationship::Stranger));
        assert!(!settings.can_receive_dm_from(Relationship::Blocked));
    }

    #[test]
    fn test_apply_preset_public() {
        let user_id = Uuid::new_v4();
        let mut settings = UserPrivacySettings::new(user_id);
        
        settings.apply_preset(PrivacyPreset::Public);
        
        assert_eq!(settings.allow_dms_from(), DmPrivacy::Everyone);
        assert!(settings.show_profile_to_non_friends());
    }

    #[test]
    fn test_apply_preset_private() {
        let user_id = Uuid::new_v4();
        let mut settings = UserPrivacySettings::new(user_id);
        
        settings.apply_preset(PrivacyPreset::Private);
        
        assert_eq!(settings.allow_dms_from(), DmPrivacy::None);
        assert!(!settings.show_online_status());
        assert!(!settings.show_profile_to_non_friends());
    }

    #[test]
    fn test_profile_visibility() {
        let user_id = Uuid::new_v4();
        let mut settings = UserPrivacySettings::new(user_id);
        
        // Default: visible to non-friends
        assert!(settings.is_profile_visible_to(Relationship::Stranger));
        
        // Friends always visible
        assert!(settings.is_profile_visible_to(Relationship::Friend));
        
        // Blocked never visible
        assert!(!settings.is_profile_visible_to(Relationship::Blocked));
        
        // Change setting
        settings.update_visibility(None, None, Some(false));
        assert!(!settings.is_profile_visible_to(Relationship::Stranger));
    }
}
