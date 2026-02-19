use std::fmt;

use super::error::GuildRoleError;

// ============================================
// PERMISSIONS (Bitfield)
// ============================================

/// Bitfield of guild permissions
///
/// Each bit represents a permission. Combine with bitwise OR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Permissions(pub i64);

impl Permissions {
    // ---- Message permissions ----
    pub const SEND_MESSAGES: i64 = 1 << 0;
    pub const READ_MESSAGE_HISTORY: i64 = 1 << 1;
    pub const EMBED_LINKS: i64 = 1 << 2;
    pub const ATTACH_FILES: i64 = 1 << 3;
    pub const MENTION_EVERYONE: i64 = 1 << 4;

    // ---- Voice permissions ----
    pub const CONNECT: i64 = 1 << 10;
    pub const SPEAK: i64 = 1 << 11;
    pub const MUTE_MEMBERS: i64 = 1 << 12;
    pub const DEAFEN_MEMBERS: i64 = 1 << 13;

    // ---- Moderation permissions ----
    pub const KICK_MEMBERS: i64 = 1 << 20;
    pub const BAN_MEMBERS: i64 = 1 << 21;
    pub const MANAGE_MESSAGES: i64 = 1 << 22;
    pub const MANAGE_ROLES: i64 = 1 << 23;
    pub const MANAGE_CHANNELS: i64 = 1 << 24;
    pub const MANAGE_GUILD: i64 = 1 << 25;

    // ---- Admin ----
    /// Admin bypasses all other permission checks
    pub const ADMINISTRATOR: i64 = 1 << 31;

    pub fn new(bits: i64) -> Self {
        Self(bits)
    }

    pub fn none() -> Self {
        Self(0)
    }

    pub fn administrator() -> Self {
        Self(Self::ADMINISTRATOR)
    }

    pub fn bits(self) -> i64 {
        self.0
    }

    pub fn has(self, permission: i64) -> bool {
        self.0 & Self::ADMINISTRATOR != 0 || self.0 & permission != 0
    }

    pub fn add(self, permission: i64) -> Self {
        Self(self.0 | permission)
    }

    pub fn remove(self, permission: i64) -> Self {
        Self(self.0 & !permission)
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================
// ROLE COLOR
// ============================================

/// Role color stored as RGB integer (0 = no color / default)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RoleColor(pub i32);

impl RoleColor {
    /// Parse from hex string, e.g. "#FF5733" or "FF5733"
    pub fn from_hex(hex: &str) -> Result<Self, GuildRoleError> {
        let hex = hex.trim_start_matches('#');
        let rgb = i32::from_str_radix(hex, 16).map_err(|_| GuildRoleError::InvalidColor)?;
        Ok(Self(rgb))
    }

    pub fn to_hex(self) -> String {
        format!("#{:06X}", self.0)
    }

    pub fn value(self) -> i32 {
        self.0
    }

    pub fn is_default(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for RoleColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let perms = Permissions::new(Permissions::SEND_MESSAGES | Permissions::READ_MESSAGE_HISTORY);
        assert!(perms.has(Permissions::SEND_MESSAGES));
        assert!(perms.has(Permissions::READ_MESSAGE_HISTORY));
        assert!(!perms.has(Permissions::ADMINISTRATOR));
    }

    #[test]
    fn test_administrator_bypasses_all() {
        let perms = Permissions::administrator();
        assert!(perms.has(Permissions::KICK_MEMBERS));
        assert!(perms.has(Permissions::BAN_MEMBERS));
        assert!(perms.has(Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_role_color() {
        let color = RoleColor::from_hex("#FF5733").unwrap();
        assert_eq!(color.to_hex(), "#FF5733");

        let invalid = RoleColor::from_hex("ZZZZZZ");
        assert!(invalid.is_err());
    }
}
