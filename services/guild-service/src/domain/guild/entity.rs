use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::GuildError;
use super::valueobject::{GuildName, GuildVisibility};

/// Guild Aggregate Root
///
/// Represents a server/community where users can communicate.
/// Owns channels, roles, members, and invite links.
#[derive(Debug, Clone)]
pub struct Guild {
    // ============================================
    // IDENTITY
    // ============================================
    id: Uuid,
    owner_id: Uuid,

    // ============================================
    // SETTINGS
    // ============================================
    name: GuildName,
    description: Option<String>,
    icon_url: Option<String>,
    banner_url: Option<String>,
    visibility: GuildVisibility,

    // ============================================
    // LIMITS
    // ============================================
    /// Cached member count (updated asynchronously)
    member_count: i32,
    /// Maximum allowed members (default: 500)
    max_members: i32,

    // ============================================
    // METADATA
    // ============================================
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Guild {
    // ============================================
    // CONSTRUCTION
    // ============================================

    /// Create a new guild
    pub fn new(
        owner_id: Uuid,
        name: GuildName,
        description: Option<String>,
    ) -> Result<Self, GuildError> {
        if let Some(ref desc) = description {
            if desc.len() > 1000 {
                return Err(GuildError::DescriptionTooLong);
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            owner_id,
            name,
            description,
            icon_url: None,
            banner_url: None,
            visibility: GuildVisibility::default(),
            member_count: 1, // Owner is the first member
            max_members: 500,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: Uuid,
        owner_id: Uuid,
        name: GuildName,
        description: Option<String>,
        icon_url: Option<String>,
        banner_url: Option<String>,
        visibility: GuildVisibility,
        member_count: i32,
        max_members: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            description,
            icon_url,
            banner_url,
            visibility,
            member_count,
            max_members,
            created_at,
            updated_at,
            deleted_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn owner_id(&self) -> Uuid {
        self.owner_id
    }

    pub fn name(&self) -> &GuildName {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn icon_url(&self) -> Option<&str> {
        self.icon_url.as_deref()
    }

    pub fn banner_url(&self) -> Option<&str> {
        self.banner_url.as_deref()
    }

    pub fn visibility(&self) -> GuildVisibility {
        self.visibility
    }

    pub fn member_count(&self) -> i32 {
        self.member_count
    }

    pub fn max_members(&self) -> i32 {
        self.max_members
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    pub fn is_owner(&self, user_id: Uuid) -> bool {
        self.owner_id == user_id
    }

    pub fn is_full(&self) -> bool {
        self.member_count >= self.max_members
    }

    // ============================================
    // BUSINESS LOGIC - Settings
    // ============================================

    /// Update guild settings
    pub fn update(
        &mut self,
        name: Option<GuildName>,
        description: Option<String>,
        icon_url: Option<String>,
        banner_url: Option<String>,
        visibility: Option<GuildVisibility>,
    ) -> Result<(), GuildError> {
        if let Some(n) = name {
            self.name = n;
        }

        if let Some(desc) = description {
            if desc.len() > 1000 {
                return Err(GuildError::DescriptionTooLong);
            }
            self.description = Some(desc);
        }

        if let Some(url) = icon_url {
            Self::validate_url(&url).map_err(|_| GuildError::InvalidIconUrl)?;
            self.icon_url = Some(url);
        }

        if let Some(url) = banner_url {
            Self::validate_url(&url).map_err(|_| GuildError::InvalidBannerUrl)?;
            self.banner_url = Some(url);
        }

        if let Some(v) = visibility {
            self.visibility = v;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Transfer ownership to another member
    ///
    /// **Important:** Caller must verify that `new_owner_id` is an active member
    pub fn transfer_ownership(&mut self, requester_id: Uuid, new_owner_id: Uuid) -> Result<(), GuildError> {
        if self.owner_id != requester_id {
            return Err(GuildError::NotOwner);
        }

        self.owner_id = new_owner_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    // ============================================
    // BUSINESS LOGIC - Member Count
    // ============================================

    /// Increment cached member count
    pub fn increment_member_count(&mut self) -> Result<(), GuildError> {
        if self.is_full() {
            return Err(GuildError::MemberLimitReached);
        }
        self.member_count += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Decrement cached member count
    pub fn decrement_member_count(&mut self) {
        self.member_count = (self.member_count - 1).max(0);
        self.updated_at = Utc::now();
    }

    // ============================================
    // BUSINESS LOGIC - Deletion
    // ============================================

    /// Soft delete the guild (owner only)
    pub fn delete(&mut self, requester_id: Uuid) -> Result<(), GuildError> {
        if self.owner_id != requester_id {
            return Err(GuildError::NotOwner);
        }
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    // ============================================
    // HELPERS
    // ============================================

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

    fn make_guild() -> Guild {
        let owner = Uuid::new_v4();
        let name = GuildName::new("Test Guild").unwrap();
        Guild::new(owner, name, None).unwrap()
    }

    #[test]
    fn test_create_guild() {
        let guild = make_guild();
        assert!(guild.is_active());
        assert_eq!(guild.member_count(), 1);
        assert_eq!(guild.max_members(), 500);
    }

    #[test]
    fn test_owner_check() {
        let owner = Uuid::new_v4();
        let name = GuildName::new("My Guild").unwrap();
        let guild = Guild::new(owner, name, None).unwrap();

        assert!(guild.is_owner(owner));
        assert!(!guild.is_owner(Uuid::new_v4()));
    }

    #[test]
    fn test_delete_requires_owner() {
        let owner = Uuid::new_v4();
        let name = GuildName::new("My Guild").unwrap();
        let mut guild = Guild::new(owner, name, None).unwrap();

        // Non-owner cannot delete
        assert!(guild.delete(Uuid::new_v4()).is_err());

        // Owner can delete
        assert!(guild.delete(owner).is_ok());
        assert!(guild.is_deleted());
    }

    #[test]
    fn test_transfer_ownership() {
        let owner = Uuid::new_v4();
        let new_owner = Uuid::new_v4();
        let name = GuildName::new("My Guild").unwrap();
        let mut guild = Guild::new(owner, name, None).unwrap();

        assert!(guild.transfer_ownership(Uuid::new_v4(), new_owner).is_err());
        assert!(guild.transfer_ownership(owner, new_owner).is_ok());
        assert_eq!(guild.owner_id(), new_owner);
    }

    #[test]
    fn test_member_count() {
        let mut guild = make_guild();
        assert_eq!(guild.member_count(), 1);

        guild.increment_member_count().unwrap();
        assert_eq!(guild.member_count(), 2);

        guild.decrement_member_count();
        assert_eq!(guild.member_count(), 1);
    }
}
