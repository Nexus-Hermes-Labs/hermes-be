use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::error::AuthSessionError;

/// Auth Session Entity
///
/// Represents an active authentication session with a refresh token.
/// Each login creates a new session.
#[derive(Debug, Clone)]
pub struct AuthSession {
    id: Uuid,
    user_id: Uuid,
    refresh_token_hash: String,

    // Session Info
    ip_address: Option<String>,
    user_agent: Option<String>,
    device_name: Option<String>,

    // Expiry
    expires_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,

    // State
    is_revoked: bool,
    revoked_at: Option<DateTime<Utc>>,

    // Metadata
    created_at: DateTime<Utc>,
}

impl AuthSession {
    /// Create new session for user_profile
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `refresh_token_hash` - SHA256 hash of the refresh token
    /// * `expiry_days` - How long the session is valid
    /// * `ip_address` - Client IP address
    /// * `user_agent` - Client user_profile agent string
    pub fn create(
        user_id: Uuid,
        refresh_token_hash: String,
        expiry_days: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id,
            refresh_token_hash,
            ip_address,
            user_agent: user_agent.clone(),
            device_name: Self::extract_device_name(&user_agent),
            expires_at: now + Duration::days(expiry_days),
            last_used_at: now,
            is_revoked: false,
            revoked_at: None,
            created_at: now,
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: Uuid,
        user_id: Uuid,
        refresh_token_hash: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        device_name: Option<String>,
        expires_at: DateTime<Utc>,
        last_used_at: DateTime<Utc>,
        is_revoked: bool,
        revoked_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            refresh_token_hash,
            ip_address,
            user_agent,
            device_name,
            expires_at,
            last_used_at,
            is_revoked,
            revoked_at,
            created_at,
        }
    }

    // ============================================
    // GETTERS
    // ============================================

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn refresh_token_hash(&self) -> &str {
        &self.refresh_token_hash
    }

    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn last_used_at(&self) -> DateTime<Utc> {
        self.last_used_at
    }

    pub fn is_revoked(&self) -> bool {
        self.is_revoked
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // ============================================
    // BUSINESS LOGIC
    // ============================================

    /// Check if session is valid (not revoked and not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_revoked && self.expires_at > Utc::now()
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    /// Ensure session is valid for use
    pub fn ensure_valid(&self) -> Result<(), AuthSessionError> {
        if self.is_revoked {
            return Err(AuthSessionError::SessionRevoked {
                revoked_at: self.revoked_at,
            });
        }

        if self.is_expired() {
            return Err(AuthSessionError::SessionExpired {
                expired_at: self.expires_at,
            });
        }

        Ok(())
    }

    /// Mark session as used (update last_used_at)
    pub fn mark_as_used(&mut self) {
        self.last_used_at = Utc::now();
    }

    /// Update last_used_at timestamp with validation
    pub fn use_session(&mut self) -> Result<(), AuthSessionError> {
        self.ensure_valid()?;
        self.mark_as_used(); 
        Ok(())
    }

    /// Revoke session (logout)
    pub fn revoke(&mut self) {
        self.is_revoked = true;
        self.revoked_at = Some(Utc::now());
    }

    /// Check if this session matches the given refresh token hash
    pub fn matches_token(&self, token_hash: &str) -> bool {
        self.refresh_token_hash == token_hash
    }

    // ============================================
    // HELPERS
    // ============================================

    /// Extract device name from user_profile agent string
    fn extract_device_name(user_agent: &Option<String>) -> Option<String> {
        user_agent.as_ref().map(|ua| {
            // Simple extraction - in production use a proper user_profile-agent parser
            if ua.contains("Chrome") {
                "Chrome Browser".to_string()
            } else if ua.contains("Firefox") {
                "Firefox Browser".to_string()
            } else if ua.contains("Safari") && !ua.contains("Chrome") {
                "Safari Browser".to_string()
            } else if ua.contains("Edge") {
                "Edge Browser".to_string()
            } else if ua.contains("Mobile") {
                "Mobile Device".to_string()
            } else {
                "Unknown Device".to_string()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let user_id = Uuid::new_v4();
        let token_hash = "hash123".to_string();

        let session = AuthSession::create(
            user_id,
            token_hash.clone(),
            30,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(session.user_id(), user_id);
        assert_eq!(session.refresh_token_hash(), &token_hash);
        assert!(session.is_valid());
        assert!(!session.is_revoked());
    }

    #[test]
    fn test_revoke_session() {
        let mut session = AuthSession::create(
            Uuid::new_v4(),
            "hash".to_string(),
            30,
            None,
            None,
        );

        session.revoke();

        assert!(session.is_revoked());
        assert!(!session.is_valid());
        assert!(session.revoked_at.is_some());
    }

    #[test]
    fn test_use_session_updates_last_used() {
        let mut session = AuthSession::create(
            Uuid::new_v4(),
            "hash".to_string(),
            30,
            None,
            None,
        );

        let original_last_used = session.last_used_at();
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.use_session().unwrap();

        assert!(session.last_used_at() > original_last_used);
    }

    #[test]
    fn test_use_revoked_session_fails() {
        let mut session = AuthSession::create(
            Uuid::new_v4(),
            "hash".to_string(),
            30,
            None,
            None,
        );

        session.revoke();

        let result = session.use_session();
        assert!(matches!(result, Err(AuthSessionError::SessionRevoked { .. })));
    }

    #[test]
    fn test_device_name_extraction() {
        let chrome_session = AuthSession::create(
            Uuid::new_v4(),
            "hash".to_string(),
            30,
            None,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string()),
        );
        assert_eq!(chrome_session.device_name(), Some("Chrome Browser"));

        let firefox_session = AuthSession::create(
            Uuid::new_v4(),
            "hash".to_string(),
            30,
            None,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0".to_string()),
        );
        assert_eq!(firefox_session.device_name(), Some("Firefox Browser"));
    }

    #[test]
    fn test_matches_token() {
        let token_hash = "specific_hash_123".to_string();
        let session = AuthSession::create(
            Uuid::new_v4(),
            token_hash.clone(),
            30,
            None,
            None,
        );

        assert!(session.matches_token(&token_hash));
        assert!(!session.matches_token("wrong_hash"));
    }
}
