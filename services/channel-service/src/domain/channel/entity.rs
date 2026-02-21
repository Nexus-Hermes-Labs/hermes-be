use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::ChannelError;
use super::valueobject::{ChannelName, ChannelType};

/// Channel Aggregate Root
///
/// Represents a text, voice, category, or announcement channel within a guild.
/// Channels can be grouped under a category channel via `parent_id`.
#[derive(Debug, Clone)]
pub struct Channel {
    // ============================================
    // IDENTITY
    // ============================================
    id: Uuid,
    guild_id: Uuid,
    parent_id: Option<Uuid>,

    // ============================================
    // SETTINGS
    // ============================================
    name: ChannelName,
    r#type: ChannelType,
    description: Option<String>,
    position: i32,

    // ============================================
    // METADATA
    // ============================================
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Channel {
    // ============================================
    // CONSTRUCTION
    // ============================================

    /// Create a new channel
    pub fn new(
        guild_id: Uuid,
        parent_id: Option<Uuid>,
        name: ChannelName,
        channel_type: ChannelType,
        description: Option<String>,
        position: i32,
    ) -> Result<Self, ChannelError> {
        if let Some(ref desc) = description {
            if desc.len() > 1024 {
                return Err(ChannelError::DescriptionTooLong);
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            guild_id,
            parent_id,
            name,
            r#type: channel_type,
            description,
            position,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Reconstruct a `Channel` from a persisted database row.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn from_persisted(
        id: Uuid,
        guild_id: Uuid,
        parent_id: Option<Uuid>,
        name: ChannelName,
        channel_type: ChannelType,
        description: Option<String>,
        position: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            guild_id,
            parent_id,
            name,
            r#type: channel_type,
            description,
            position,
            created_at,
            updated_at,
            deleted_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    /// Returns the channel's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the guild this channel belongs to.
    #[must_use]
    pub const fn guild_id(&self) -> Uuid {
        self.guild_id
    }

    /// Returns the parent category channel ID, if any.
    #[must_use]
    pub const fn parent_id(&self) -> Option<Uuid> {
        self.parent_id
    }

    /// Returns the validated channel name.
    #[must_use]
    pub const fn name(&self) -> &ChannelName {
        &self.name
    }

    /// Returns the channel type.
    #[must_use]
    pub const fn channel_type(&self) -> ChannelType {
        self.r#type
    }

    /// Returns the channel description, if one has been set.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the channel's display position within the guild.
    #[must_use]
    pub const fn position(&self) -> i32 {
        self.position
    }

    /// Returns the timestamp when the channel was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the timestamp of the most recent channel update.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns `true` if the channel has been soft-deleted.
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Returns `true` if the channel has **not** been deleted.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    // ============================================
    // BUSINESS LOGIC
    // ============================================

    /// Update channel settings.
    ///
    /// Only fields wrapped in `Some` are changed; `None` fields are left unchanged.
    pub fn update(
        &mut self,
        name: Option<ChannelName>,
        parent_id: Option<Uuid>,
        description: Option<String>,
        position: Option<i32>,
    ) -> Result<(), ChannelError> {
        if let Some(n) = name {
            self.name = n;
        }

        if let Some(pid) = parent_id {
            if pid == self.id {
                return Err(ChannelError::InvalidParentId);
            }
            self.parent_id = Some(pid);
        }

        if let Some(desc) = description {
            if desc.len() > 1024 {
                return Err(ChannelError::DescriptionTooLong);
            }
            self.description = Some(desc);
        }

        if let Some(pos) = position {
            self.position = pos;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Soft-delete the channel.
    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_channel() -> Channel {
        let name = ChannelName::new("general").unwrap();
        Channel::new(Uuid::new_v4(), None, name, ChannelType::Text, None, 0).unwrap()
    }

    #[test]
    fn test_create_channel() {
        let ch = make_channel();
        assert!(ch.is_active());
        assert_eq!(ch.channel_type(), ChannelType::Text);
        assert_eq!(ch.position(), 0);
        assert!(ch.parent_id().is_none());
    }

    #[test]
    fn test_update_channel() {
        let mut ch = make_channel();
        let new_name = ChannelName::new("announcements").unwrap();
        ch.update(Some(new_name), None, None, Some(5))
            .unwrap();
        assert_eq!(ch.name().as_str(), "announcements");
        assert_eq!(ch.position(), 5);
    }

    #[test]
    fn test_update_invalid_parent() {
        let mut ch = make_channel();
        let self_id = ch.id();
        let result = ch.update(None, Some(self_id), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_soft_delete() {
        let mut ch = make_channel();
        assert!(ch.is_active());
        ch.delete();
        assert!(ch.is_deleted());
    }
}
