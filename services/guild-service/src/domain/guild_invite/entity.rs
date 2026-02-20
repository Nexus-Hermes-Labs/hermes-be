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
    #[must_use]
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
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn from_persisted(
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

    /// Returns the invite code value object.
    #[must_use]
    pub const fn code(&self) -> &InviteCode {
        &self.code
    }

    /// Returns the ID of the guild this invite grants access to.
    #[must_use]
    pub const fn guild_id(&self) -> Uuid {
        self.guild_id
    }

    /// Returns the ID of the member who created the invite.
    #[must_use]
    pub const fn creator_id(&self) -> Uuid {
        self.creator_id
    }

    /// Returns the maximum number of allowed uses, or `None` for unlimited.
    #[must_use]
    pub const fn max_uses(&self) -> Option<i32> {
        self.max_uses
    }

    /// Returns the number of times this invite has been used so far.
    #[must_use]
    pub const fn uses(&self) -> i32 {
        self.uses
    }

    /// Returns the expiry timestamp, or `None` if the invite never expires.
    #[must_use]
    pub const fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    /// Returns `true` if the invite has been explicitly revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked
    }

    /// Returns the timestamp when the invite was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // ============================================
    // BUSINESS LOGIC
    // ============================================

    /// Returns `true` if the invite can still be used (not revoked, not expired, not exhausted).
    ///
    /// Use this for read-only checks. To actually consume a use, call [`use_invite`](Self::use_invite).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired() && !self.is_exhausted()
    }

    /// Returns `true` if the invite's expiry timestamp is in the past.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| exp <= Utc::now())
    }

    /// Returns `true` if `uses` has reached `max_uses`.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.max_uses.is_some_and(|max| self.uses >= max)
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
