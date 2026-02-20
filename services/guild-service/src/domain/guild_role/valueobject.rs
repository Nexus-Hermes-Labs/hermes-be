use std::fmt;

use super::error::GuildRoleError;

// ============================================
// PERMISSIONS (Bitfield System)
// ============================================

/// Represents a set of server (guild) permissions stored as a 64-bit integer bitfield.
///
/// ### How it works:
/// Each bit in the `i64` represents a specific permission (e.g., bit 0 for sending messages,
/// bit 21 for banning members). This allows storing up to 64 distinct permissions
/// in a single efficient number.
///
/// ### Bitwise Operations:
/// - **OR (`|`)**: Combines permissions (e.g., `READ | WRITE`).
/// - **AND (`&`)**: Checks for a permission.
/// - **NOT (`!`)**: Used to remove a permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Permissions(pub i64);

impl Permissions {
    // ---- Message Permissions (Bits 0-9) ----
    /// Allows sending text messages in channels.
    pub const SEND_MESSAGES: i64 = 1 << 0;
    /// Allows reading the history of previous messages in a channel.
    pub const READ_MESSAGE_HISTORY: i64 = 1 << 1;
    /// Allows links to be previewed with rich content (embeds).
    pub const EMBED_LINKS: i64 = 1 << 2;
    /// Allows uploading and sending files/images.
    pub const ATTACH_FILES: i64 = 1 << 3;
    /// Allows using `@everyone` or `@here` to notify all members.
    pub const MENTION_EVERYONE: i64 = 1 << 4;

    // ---- Voice Permissions (Bits 10-19) ----
    /// Allows connecting to voice channels.
    pub const CONNECT: i64 = 1 << 10;
    /// Allows speaking in voice channels.
    pub const SPEAK: i64 = 1 << 11;
    /// Allows muting other members in voice channels.
    pub const MUTE_MEMBERS: i64 = 1 << 12;
    /// Allows deafening (muting output) of other members.
    pub const DEAFEN_MEMBERS: i64 = 1 << 13;

    // ---- Moderation & Management (Bits 20-30) ----
    /// Allows removing members from the server.
    pub const KICK_MEMBERS: i64 = 1 << 20;
    /// Allows permanently banning members from the server.
    pub const BAN_MEMBERS: i64 = 1 << 21;
    /// Allows deleting or pinning messages sent by others.
    pub const MANAGE_MESSAGES: i64 = 1 << 22;
    /// Allows creating, editing, and deleting roles below their own.
    pub const MANAGE_ROLES: i64 = 1 << 23;
    /// Allows creating, editing, and deleting channels.
    pub const MANAGE_CHANNELS: i64 = 1 << 24;
    /// Allows editing server settings (name, icon, etc.).
    pub const MANAGE_GUILD: i64 = 1 << 25;

    // ---- Special Permissions ----
    /// Grant all permissions and bypasses all specific checks.
    /// Use with extreme caution.
    pub const ADMINISTRATOR: i64 = 1 << 31;

    /// Creates a new `Permissions` instance from a raw bitfield value.
    pub fn new(bits: i64) -> Self {
        Self(bits)
    }

    /// Returns a `Permissions` object with zero bits set (all denied).
    pub fn none() -> Self {
        Self(0)
    }

    /// Returns a `Permissions` object with only the [`ADMINISTRATOR`] bit set.
    pub fn administrator() -> Self {
        Self(Self::ADMINISTRATOR)
    }

    /// Returns the underlying raw `i64` value.
    pub fn bits(self) -> i64 {
        self.0
    }

    /// Checks if a specific permission is granted.
    ///
    /// This method is **Safe**: It automatically returns `true` if the user has the
    /// [`ADMINISTRATOR`] permission, even if the specific bit is not set.
    ///
    /// ### Example:
    /// ```
    /// let p = Permissions::new(Permissions::BAN_MEMBERS);
    /// assert!(p.has(Permissions::BAN_MEMBERS));
    /// ```
    pub fn has(self, permission: i64) -> bool {
        (self.0 & Self::ADMINISTRATOR != 0) || (self.0 & permission != 0)
    }

    /// Grants a permission by setting its bit. Returns a new instance.
    pub fn add(self, permission: i64) -> Self {
        Self(self.0 | permission)
    }

    /// Revokes a permission by clearing its bit. Returns a new instance.
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

    /// Convert the color to an uppercase hex string, e.g. `"#FF5733"`.
    pub fn to_hex(self) -> String {
        format!("#{:06X}", self.0)
    }

    /// Return the raw RGB integer value.
    pub fn value(self) -> i32 {
        self.0
    }

    /// Returns `true` when no color has been set (value is 0 = default grey).
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
