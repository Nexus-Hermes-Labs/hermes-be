use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::UserProfileError;
use super::valueobject::{Username, UserStatus};

/// User Profile Aggregate Root
///
/// Represents a user's public profile and presence information.
/// Simplified identity system with unique usernames.
#[derive(Debug, Clone)]
pub struct UserProfile {
    // ============================================
    // IDENTITY
    // ============================================
    id: Uuid,
    username: Username,           // Unique (e.g., "johndoe")
    display_name: String,         // Non-unique (e.g., "John Doe 🎮")
    
    // ============================================
    // PROFILE
    // ============================================
    avatar_url: Option<String>,
    banner_url: Option<String>,
    bio: Option<String>,
    
    // ============================================
    // PRESENCE
    // ============================================
    status: UserStatus,
    custom_status_text: Option<String>,
    custom_status_emoji: Option<String>,
    custom_status_expires_at: Option<DateTime<Utc>>,
    last_seen_at: Option<DateTime<Utc>>,
    
    // ============================================
    // METADATA
    // ============================================
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    
    // Username change tracking (rate limiting)
    last_username_changed_at: Option<DateTime<Utc>>,
}

impl UserProfile {
    // ============================================
    // CONSTRUCTION
    // ============================================
    
    /// Create new user profile
    pub fn new(
        username: Username,
        display_name: String,
    ) -> Result<Self, UserProfileError> {
        // Validate display name
        if display_name.is_empty() || display_name.len() > 100 {
            return Err(UserProfileError::InvalidDisplayName);
        }
        
        let now = Utc::now();
        
        Ok(Self {
            id: Uuid::new_v4(),
            username,
            display_name,
            avatar_url: None,
            banner_url: None,
            bio: None,
            status: UserStatus::default(),
            custom_status_text: None,
            custom_status_emoji: None,
            custom_status_expires_at: None,
            last_seen_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            last_username_changed_at: None,
        })
    }
    
    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: Uuid,
        username: Username,
        display_name: String,
        avatar_url: Option<String>,
        banner_url: Option<String>,
        bio: Option<String>,
        status: UserStatus,
        custom_status_text: Option<String>,
        custom_status_emoji: Option<String>,
        custom_status_expires_at: Option<DateTime<Utc>>,
        last_seen_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
        last_username_changed_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            username,
            display_name,
            avatar_url,
            banner_url,
            bio,
            status,
            custom_status_text,
            custom_status_emoji,
            custom_status_expires_at,
            last_seen_at,
            created_at,
            updated_at,
            deleted_at,
            last_username_changed_at,
        }
    }
    
    // ============================================
    // GETTERS
    // ============================================
    
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    pub fn username(&self) -> &Username {
        &self.username
    }
    
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    
    /// Get full display format: "Display Name (@username)"
    pub fn full_display(&self) -> String {
        format!("{} (@{})", self.display_name, self.username)
    }
    
    /// Get mention format: "@username"
    pub fn mention(&self) -> String {
        self.username.mention()
    }
    
    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_deref()
    }
    
    pub fn banner_url(&self) -> Option<&str> {
        self.banner_url.as_deref()
    }
    
    pub fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }
    
    pub fn status(&self) -> UserStatus {
        self.status
    }
    
    pub fn custom_status_text(&self) -> Option<&str> {
        self.custom_status_text.as_deref()
    }
    
    pub fn custom_status_emoji(&self) -> Option<&str> {
        self.custom_status_emoji.as_deref()
    }
    
    pub fn custom_status_expires_at(&self) -> Option<DateTime<Utc>> {
        self.custom_status_expires_at
    }
    
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    
    pub fn is_online(&self) -> bool {
        self.status.is_online()
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    pub fn last_seen_at(&self) -> Option<DateTime<Utc>> {
        self.last_seen_at
    }
    
    // ============================================
    // BUSINESS LOGIC - Profile Updates
    // ============================================
    
    /// Update profile information
    pub fn update_profile(
        &mut self,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        banner_url: Option<String>,
    ) -> Result<(), UserProfileError> {
        if let Some(name) = display_name {
            Self::validate_display_name(&name)?;
            self.display_name = name;
        }

        if let Some(bio_text) = bio {
            Self::validate_bio(&bio_text)?;
            self.bio = Some(bio_text);
        }

        if let Some(url) = avatar_url {
            Self::validate_url(&url)
                .map_err(|_| UserProfileError::InvalidAvatarUrl)?;
            self.avatar_url = Some(url);
        }

        if let Some(url) = banner_url {
            Self::validate_url(&url)
                .map_err(|_| UserProfileError::InvalidBannerUrl)?;
            self.banner_url = Some(url);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Change username
    ///
    /// Business rule: Can only change username once per 30 days
    ///
    /// **Important:** Repository must verify username is available
    /// before calling this method!
    pub fn change_username(&mut self, new_username: Username) -> Result<(), UserProfileError> {
        // Rate limit: 30 days
        if let Some(last_change) = self.last_username_changed_at {
            let days_since_change = Utc::now().signed_duration_since(last_change).num_days();
            if days_since_change < 30 {
                return Err(UserProfileError::UsernameChangeRateLimited);
            }
        }
        
        self.username = new_username;
        self.last_username_changed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    // ============================================
    // BUSINESS LOGIC - Presence
    // ============================================
    
    /// Update presence status
    pub fn update_status(&mut self, status: UserStatus) {
        self.status = status;
        self.last_seen_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Set custom status
    pub fn set_custom_status(
        &mut self,
        text: Option<String>,
        emoji: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), UserProfileError> {
        // Clear custom status if both text and emoji are None
        if text.is_none() && emoji.is_none() {
            self.custom_status_text = None;
            self.custom_status_emoji = None;
            self.custom_status_expires_at = None;
            self.updated_at = Utc::now();
            return Ok(());
        }
        
        // Validate text length
        if let Some(ref t) = text {
            if t.len() > 128 {
                return Err(UserProfileError::CustomStatusTooLong);
            }
        }
        
        // Validate emoji length
        if let Some(ref e) = emoji {
            if e.len() > 50 {
                return Err(UserProfileError::CustomStatusEmojiTooLong);
            }
        }
        
        // Validate expiration
        if let Some(exp) = expires_at {
            if exp <= Utc::now() {
                return Err(UserProfileError::CustomStatusExpirationInPast);
            }
        }
        
        self.custom_status_text = text;
        self.custom_status_emoji = emoji;
        self.custom_status_expires_at = expires_at;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Clear custom status
    pub fn clear_custom_status(&mut self) {
        self.custom_status_text = None;
        self.custom_status_emoji = None;
        self.custom_status_expires_at = None;
        self.updated_at = Utc::now();
    }
    
    /// Check if custom status has expired
    pub fn has_custom_status_expired(&self) -> bool {
        if let Some(expires_at) = self.custom_status_expires_at {
            expires_at <= Utc::now()
        } else {
            false
        }
    }
    
    // ============================================
    // BUSINESS LOGIC - Account Management
    // ============================================
    
    /// Soft delete profile
    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Check if profile is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
    
    // ============================================
    // HELPERS
    // ============================================

    fn validate_display_name(name: &str) -> Result<(), UserProfileError> {
        if name.is_empty() {
            return Err(UserProfileError::InvalidDisplayName);
        }
        if name.len() > 100 {
            return Err(UserProfileError::DisplayNameTooLong);
        }
        Ok(())
    }

    fn validate_bio(bio: &str) -> Result<(), UserProfileError> {
        if bio.len() > 500 {
            return Err(UserProfileError::BioTooLong);
        }
        Ok(())
    }

    fn validate_url(url: &str) -> Result<(), ()> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_profile() {
        let username = Username::new("johndoe").unwrap();
        
        let profile = UserProfile::new(username.clone(), "John Doe".to_string()).unwrap();
        
        assert_eq!(profile.username(), &username);
        assert_eq!(profile.display_name(), "John Doe");
        assert!(profile.is_active());
    }

    #[test]
    fn test_full_display_and_mention() {
        let username = Username::new("johndoe").unwrap();
        let profile = UserProfile::new(username, "John Doe 🎮".to_string()).unwrap();
        
        assert_eq!(profile.full_display(), "John Doe 🎮 (@johndoe)");
        assert_eq!(profile.mention(), "@johndoe");
    }

    #[test]
    fn test_update_profile() {
        let username = Username::new("johndoe").unwrap();
        let mut profile = UserProfile::new(username, "John Doe".to_string()).unwrap();
        
        profile.update_profile(
            Some("Jane Doe".to_string()),
            Some("Hello world!".to_string()),
            None,
            None,
        ).unwrap();
        
        assert_eq!(profile.display_name(), "Jane Doe");
        assert_eq!(profile.bio(), Some("Hello world!"));
    }

    #[test]
    fn test_change_username_rate_limit() {
        let username = Username::new("johndoe").unwrap();
        let mut profile = UserProfile::new(username, "John Doe".to_string()).unwrap();
        
        // First change should work
        let new_username = Username::new("janedoe").unwrap();
        assert!(profile.change_username(new_username).is_ok());
        
        // Second change immediately should fail
        let another_username = Username::new("testuser").unwrap();
        assert!(profile.change_username(another_username).is_err());
    }

    #[test]
    fn test_custom_status() {
        let username = Username::new("johndoe").unwrap();
        let mut profile = UserProfile::new(username, "John Doe".to_string()).unwrap();
        
        profile.set_custom_status(
            Some("Coding".to_string()),
            Some("💻".to_string()),
            None,
        ).unwrap();
        
        assert_eq!(profile.custom_status_text(), Some("Coding"));
        assert_eq!(profile.custom_status_emoji(), Some("💻"));
    }

    #[test]
    fn test_delete_profile() {
        let username = Username::new("johndoe").unwrap();
        let mut profile = UserProfile::new(username, "John Doe".to_string()).unwrap();
        
        assert!(profile.is_active());
        
        profile.delete();
        
        assert!(!profile.is_active());
        assert!(profile.is_deleted());
    }
}
