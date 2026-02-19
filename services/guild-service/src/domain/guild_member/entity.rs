use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::GuildMemberError;
use super::valueobject::MemberNickname;

/// Guild Member Aggregate Root
///
/// Represents a user's membership in a specific guild.
/// A member can have multiple roles and an optional nickname.
#[derive(Debug, Clone)]
pub struct GuildMember {
    // ============================================
    // IDENTITY
    // ============================================
    guild_id: Uuid,
    user_id: Uuid,

    // ============================================
    // DISPLAY
    // ============================================
    /// Server-specific nickname (overrides username if set)
    nickname: Option<MemberNickname>,

    // ============================================
    // ROLES
    // ============================================
    role_ids: Vec<Uuid>,

    // ============================================
    // METADATA
    // ============================================
    joined_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    /// Set when member is kicked/banned/leaves
    left_at: Option<DateTime<Utc>>,
}

impl GuildMember {
    // ============================================
    // CONSTRUCTION
    // ============================================

    /// Create a new guild member (when user joins)
    pub fn new(guild_id: Uuid, user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            guild_id,
            user_id,
            nickname: None,
            role_ids: Vec::new(),
            joined_at: now,
            updated_at: now,
            left_at: None,
        }
    }

    /// Reconstruct from database
    pub fn from_persisted(
        guild_id: Uuid,
        user_id: Uuid,
        nickname: Option<MemberNickname>,
        role_ids: Vec<Uuid>,
        joined_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        left_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            guild_id,
            user_id,
            nickname,
            role_ids,
            joined_at,
            updated_at,
            left_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    pub fn guild_id(&self) -> Uuid {
        self.guild_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn nickname(&self) -> Option<&MemberNickname> {
        self.nickname.as_ref()
    }

    pub fn role_ids(&self) -> &[Uuid] {
        &self.role_ids
    }

    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_active(&self) -> bool {
        self.left_at.is_none()
    }

    pub fn has_role(&self, role_id: Uuid) -> bool {
        self.role_ids.contains(&role_id)
    }

    // ============================================
    // BUSINESS LOGIC - Nickname
    // ============================================

    /// Set or clear member nickname
    pub fn set_nickname(&mut self, nickname: Option<MemberNickname>) {
        self.nickname = nickname;
        self.updated_at = Utc::now();
    }

    // ============================================
    // BUSINESS LOGIC - Roles
    // ============================================

    /// Assign a role to this member
    pub fn assign_role(&mut self, role_id: Uuid) -> Result<(), GuildMemberError> {
        if self.role_ids.contains(&role_id) {
            return Err(GuildMemberError::RoleAlreadyAssigned);
        }
        self.role_ids.push(role_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove a role from this member
    pub fn remove_role(&mut self, role_id: Uuid) -> Result<(), GuildMemberError> {
        let pos = self
            .role_ids
            .iter()
            .position(|&id| id == role_id)
            .ok_or(GuildMemberError::RoleNotAssigned)?;
        self.role_ids.swap_remove(pos);
        self.updated_at = Utc::now();
        Ok(())
    }

    // ============================================
    // BUSINESS LOGIC - Leave / Kick
    // ============================================

    /// Mark member as left (leave or kick)
    pub fn leave(&mut self) {
        self.left_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_creation() {
        let member = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(member.is_active());
        assert!(member.role_ids().is_empty());
    }

    #[test]
    fn test_role_management() {
        let mut member = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        let role_id = Uuid::new_v4();

        assert!(member.assign_role(role_id).is_ok());
        assert!(member.has_role(role_id));
        assert!(member.assign_role(role_id).is_err()); // Duplicate

        assert!(member.remove_role(role_id).is_ok());
        assert!(!member.has_role(role_id));
        assert!(member.remove_role(role_id).is_err()); // Not assigned
    }

    #[test]
    fn test_leave() {
        let mut member = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(member.is_active());
        member.leave();
        assert!(!member.is_active());
    }
}
