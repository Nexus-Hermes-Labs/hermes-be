use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::GuildInviteError;
use super::valueobject::InviteCode;

/// Guild Invite Aggregate Root
///
/// A shareable code that allows users to join a guild.
/// Can have expiry time and/or max-uses limit.
#[derive(Debug, Clone)]
pub struct GuildInvite {
    code: InviteCode,
    guild_id: Uuid,
    creator_id: Uuid,

    /// None = unlimited
    max_uses: Option<i32>,
    uses: i32,

    /// None = never expires
    expires_at: Option<DateTime<Utc>>,
    revoked: bool,

    created_at: DateTime<Utc>,
}

impl GuildInvite {
    // ============================================
    // CONSTRUCTION
    // ============================================

    /// Create a new invite
    pub fn new(
        guild_id: Uuid,
        creator_id: Uuid,
        max_uses: Option<i32>,
        max_age_seconds: Option<i64>,
    ) -> Self {
        let expires_at = max_age_seconds
            .filter(|&s| s > 0)
            .map(|s| Utc::now() + chrono::Duration::seconds(s));

        Self {
            code: InviteCode::generate(),
            guild_id,
            creator_id,
            max_uses,
            uses: 0,
            expires_at,
            revoked: false,
            created_at: Utc::now(),
        }
    }

    /// Reconstruct from database
    pub fn from_persisted(
        code: InviteCode,
        guild_id: Uuid,
        creator_id: Uuid,
        max_uses: Option<i32>,
        uses: i32,
        expires_at: Option<DateTime<Utc>>,
        revoked: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            code,
            guild_id,
            creator_id,
            max_uses,
            uses,
            expires_at,
            revoked,
            created_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    pub fn code(&self) -> &InviteCode {
        &self.code
    }

    pub fn guild_id(&self) -> Uuid {
        self.guild_id
    }

    pub fn creator_id(&self) -> Uuid {
        self.creator_id
    }

    pub fn max_uses(&self) -> Option<i32> {
        self.max_uses
    }

    pub fn uses(&self) -> i32 {
        self.uses
    }

    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // ============================================
    // BUSINESS LOGIC
    // ============================================

    /// Check validity without consuming a use
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired() && !self.is_exhausted()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Utc::now())
    }

    pub fn is_exhausted(&self) -> bool {
        self.max_uses.map_or(false, |max| self.uses >= max)
    }

    /// Use the invite (join the guild)
    pub fn use_invite(&mut self) -> Result<(), GuildInviteError> {
        if self.revoked {
            return Err(GuildInviteError::AlreadyRevoked);
        }
        if self.is_expired() {
            return Err(GuildInviteError::Expired);
        }
        if self.is_exhausted() {
            return Err(GuildInviteError::MaxUsesReached);
        }
        self.uses += 1;
        Ok(())
    }

    /// Revoke this invite
    pub fn revoke(&mut self, requester_id: Uuid) -> Result<(), GuildInviteError> {
        if self.revoked {
            return Err(GuildInviteError::AlreadyRevoked);
        }
        // Only creator can revoke (guild admins are checked at the service layer)
        if self.creator_id != requester_id {
            return Err(GuildInviteError::Unauthorized);
        }
        self.revoked = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_creation() {
        let invite = GuildInvite::new(Uuid::new_v4(), Uuid::new_v4(), Some(10), None);
        assert!(invite.is_valid());
        assert_eq!(invite.uses(), 0);
    }

    #[test]
    fn test_use_invite() {
        let mut invite = GuildInvite::new(Uuid::new_v4(), Uuid::new_v4(), Some(2), None);
        assert!(invite.use_invite().is_ok());
        assert!(invite.use_invite().is_ok());
        assert!(invite.use_invite().is_err()); // Max uses reached
    }

    #[test]
    fn test_revoke_invite() {
        let creator = Uuid::new_v4();
        let mut invite = GuildInvite::new(Uuid::new_v4(), creator, None, None);

        assert!(invite.revoke(Uuid::new_v4()).is_err()); // Unauthorized
        assert!(invite.revoke(creator).is_ok());
        assert!(invite.revoke(creator).is_err()); // Already revoked
    }
}
