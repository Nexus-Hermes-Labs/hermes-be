use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::GuildRoleError;
use super::valueobject::{Permissions, RoleColor};

/// Guild Role Aggregate Root
///
/// Defines a permission set that can be assigned to members.
/// Every guild has a default "@everyone" role (position 0).
#[derive(Debug, Clone)]
pub struct GuildRole {
    id: Uuid,
    guild_id: Uuid,
    name: String,
    color: RoleColor,
    permissions: Permissions,
    /// Display position (higher = more visible, rendered first)
    position: i32,
    /// Whether to display this role separately in the member list
    hoist: bool,
    /// Whether this role is mentionable by everyone
    mentionable: bool,
    /// True for the auto-created @everyone role
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl GuildRole {
    // ============================================
    // CONSTRUCTION
    // ============================================

    /// Create a new custom role
    pub fn new(
        guild_id: Uuid,
        name: String,
        permissions: Permissions,
        position: i32,
    ) -> Result<Self, GuildRoleError> {
        Self::validate_name(&name)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            guild_id,
            name,
            color: RoleColor::default(),
            permissions,
            position,
            hoist: false,
            mentionable: false,
            is_default: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create the default @everyone role for a new guild
    pub fn new_everyone(guild_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            guild_id,
            name: "@everyone".to_string(),
            color: RoleColor::default(),
            permissions: Permissions::new(
                Permissions::SEND_MESSAGES
                    | Permissions::READ_MESSAGE_HISTORY
                    | Permissions::EMBED_LINKS
                    | Permissions::ATTACH_FILES
                    | Permissions::CONNECT
                    | Permissions::SPEAK,
            ),
            position: 0,
            hoist: false,
            mentionable: true,
            is_default: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: Uuid,
        guild_id: Uuid,
        name: String,
        color: RoleColor,
        permissions: Permissions,
        position: i32,
        hoist: bool,
        mentionable: bool,
        is_default: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            guild_id,
            name,
            color,
            permissions,
            position,
            hoist,
            mentionable,
            is_default,
            created_at,
            updated_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn guild_id(&self) -> Uuid {
        self.guild_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> RoleColor {
        self.color
    }

    pub fn permissions(&self) -> Permissions {
        self.permissions
    }

    pub fn position(&self) -> i32 {
        self.position
    }

    pub fn hoist(&self) -> bool {
        self.hoist
    }

    pub fn mentionable(&self) -> bool {
        self.mentionable
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ============================================
    // BUSINESS LOGIC
    // ============================================

    /// Update role settings
    pub fn update(
        &mut self,
        name: Option<String>,
        color: Option<RoleColor>,
        permissions: Option<Permissions>,
        hoist: Option<bool>,
        mentionable: Option<bool>,
    ) -> Result<(), GuildRoleError> {
        if let Some(n) = name {
            Self::validate_name(&n)?;
            self.name = n;
        }
        if let Some(c) = color {
            self.color = c;
        }
        if let Some(p) = permissions {
            self.permissions = p;
        }
        if let Some(h) = hoist {
            self.hoist = h;
        }
        if let Some(m) = mentionable {
            self.mentionable = m;
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update role position
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    // ============================================
    // HELPERS
    // ============================================

    fn validate_name(name: &str) -> Result<(), GuildRoleError> {
        if name.is_empty() || name.len() > 100 {
            return Err(GuildRoleError::InvalidNameLength);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role() {
        let role = GuildRole::new(
            Uuid::new_v4(),
            "Moderator".to_string(),
            Permissions::new(Permissions::KICK_MEMBERS | Permissions::BAN_MEMBERS),
            1,
        )
        .unwrap();

        assert_eq!(role.name(), "Moderator");
        assert!(role.permissions().has(Permissions::KICK_MEMBERS));
        assert!(!role.is_default());
    }

    #[test]
    fn test_everyone_role() {
        let role = GuildRole::new_everyone(Uuid::new_v4());
        assert_eq!(role.name(), "@everyone");
        assert!(role.is_default());
        assert_eq!(role.position(), 0);
        assert!(role.mentionable());
    }
}
