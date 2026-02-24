use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::AuthCredentialError;
use super::valueobject::{AccountStatus, Email, PasswordHash, SystemRole};

/// Auth Credential Entity
///
/// Represents authentication credentials and security state.
/// This is the Aggregate Root for authentication in the auth-service.
#[derive(Debug, Clone)]
pub struct AuthCredential {
    id: Uuid,
    user_id: Uuid,
    email: Email,
    password_hash: PasswordHash,

    // Email Verification
    email_verified: bool,
    email_verification_token: Option<String>,
    email_verification_expires_at: Option<DateTime<Utc>>,

    // Security
    failed_login_attempts: i32,
    locked_until: Option<DateTime<Utc>>,
    last_login_at: Option<DateTime<Utc>>,
    last_login_ip: Option<String>,

    // Account State
    account_status: AccountStatus,
    system_role: SystemRole,
    deleted_at: Option<DateTime<Utc>>,

    // Password Management
    password_reset_token: Option<String>,
    password_reset_expires_at: Option<DateTime<Utc>>,
    password_changed_at: Option<DateTime<Utc>>,

    // Metadata
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AuthCredential {
    /// Create new auth credential for registration
    pub fn new(user_id: Uuid, email: Email, password_hash: PasswordHash) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id,
            email,
            password_hash,
            email_verified: false,
            email_verification_token: None,
            email_verification_expires_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: None,
            last_login_ip: None,
            account_status: AccountStatus::Active,
            system_role: SystemRole::User,
            deleted_at: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            password_changed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: Uuid,
        user_id: Uuid,
        email: Email,
        password_hash: PasswordHash,
        email_verified: bool,
        email_verification_token: Option<String>,
        email_verification_expires_at: Option<DateTime<Utc>>,
        failed_login_attempts: i32,
        locked_until: Option<DateTime<Utc>>,
        last_login_at: Option<DateTime<Utc>>,
        last_login_ip: Option<String>,
        account_status: AccountStatus,
        system_role: SystemRole,
        deleted_at: Option<DateTime<Utc>>,
        password_reset_token: Option<String>,
        password_reset_expires_at: Option<DateTime<Utc>>,
        password_changed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            email,
            password_hash,
            email_verified,
            email_verification_token,
            email_verification_expires_at,
            failed_login_attempts,
            locked_until,
            last_login_at,
            last_login_ip,
            account_status,
            system_role,
            deleted_at,
            password_reset_token,
            password_reset_expires_at,
            password_changed_at,
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

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn is_email_verified(&self) -> bool {
        self.email_verified
    }

    pub fn email_verification_token(&self) -> Option<&str> {
        self.email_verification_token.as_deref()
    }

    pub fn email_verification_expires_at(&self) -> Option<DateTime<Utc>> {
        self.email_verification_expires_at
    }

    pub fn failed_login_attempts(&self) -> i32 {
        self.failed_login_attempts
    }

    pub fn locked_until(&self) -> Option<DateTime<Utc>> {
        self.locked_until
    }

    pub fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }

    pub fn last_login_ip(&self) -> Option<&str> {
        self.last_login_ip.as_deref()
    }

    pub fn account_status(&self) -> &AccountStatus {
        &self.account_status
    }

    pub fn system_role(&self) -> SystemRole {
        self.system_role
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    pub fn password_reset_token(&self) -> Option<&str> {
        self.password_reset_token.as_deref()
    }

    pub fn password_reset_expires_at(&self) -> Option<DateTime<Utc>> {
        self.password_reset_expires_at
    }

    pub fn password_changed_at(&self) -> Option<DateTime<Utc>> {
        self.password_changed_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ============================================
    // BUSINESS LOGIC - EMAIL VERIFICATION
    // ============================================

    /// Generate email verification token
    pub fn generate_verification_token(&mut self) -> String {
        use rand::Rng;
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        self.email_verification_token = Some(token.clone());
        self.email_verification_expires_at = Some(Utc::now() + chrono::Duration::hours(24));
        self.updated_at = Utc::now();

        token
    }

    /// Verify email with token
    pub fn verify_email(&mut self, token: &str) -> Result<(), AuthCredentialError> {
        // Check if token matches
        let stored_token = self
            .email_verification_token
            .as_ref()
            .ok_or(AuthCredentialError::NoVerificationToken)?;

        if stored_token != token {
            return Err(AuthCredentialError::InvalidVerificationToken);
        }

        // Check if token is expired
        if let Some(expires_at) = self.email_verification_expires_at {
            if expires_at < Utc::now() {
                return Err(AuthCredentialError::VerificationTokenExpired);
            }
        }

        // Mark as verified
        self.email_verified = true;
        self.email_verification_token = None;
        self.email_verification_expires_at = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    // ============================================
    // BUSINESS LOGIC - PASSWORD RESET
    // ============================================

    /// Generate password reset token
    pub fn generate_password_reset_token(&mut self) -> String {
        use rand::Rng;
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        self.password_reset_token = Some(token.clone());
        self.password_reset_expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        self.updated_at = Utc::now();

        token
    }

    /// Reset password with token
    pub fn reset_password(
        &mut self,
        token: &str,
        new_password_hash: PasswordHash,
    ) -> Result<(), AuthCredentialError> {
        // Check if token matches
        let stored_token = self
            .password_reset_token
            .as_ref()
            .ok_or(AuthCredentialError::NoPasswordResetToken)?;

        if stored_token != token {
            return Err(AuthCredentialError::InvalidPasswordResetToken);
        }

        // Check if token is expired
        if let Some(expires_at) = self.password_reset_expires_at {
            if expires_at < Utc::now() {
                return Err(AuthCredentialError::PasswordResetTokenExpired);
            }
        }

        // Update password
        self.password_hash = new_password_hash;
        self.password_reset_token = None;
        self.password_reset_expires_at = None;
        self.password_changed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Change password (without token - for authenticated users)
    pub fn change_password(&mut self, new_password_hash: PasswordHash) {
        self.password_hash = new_password_hash;
        self.password_changed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    // ============================================
    // BUSINESS LOGIC - LOGIN ATTEMPTS
    // ============================================

    /// Record successful login
    pub fn record_successful_login(&mut self, ip_address: Option<String>) {
        self.failed_login_attempts = 0;
        self.locked_until = None;
        self.last_login_at = Some(Utc::now());
        self.last_login_ip = ip_address;
        self.updated_at = Utc::now();
    }

    /// Record failed login attempt
    ///
    /// Returns true if account should be locked
    pub fn record_failed_login(&mut self) -> bool {
        self.failed_login_attempts += 1;
        self.updated_at = Utc::now();

        // Lock account after 5 failed attempts for 15 minutes
        if self.failed_login_attempts >= 5 {
            self.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));
            true
        } else {
            false
        }
    }

    /// Check if account is currently locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            locked_until > Utc::now()
        } else {
            false
        }
    }

    /// Manually unlock account (admin action)
    pub fn unlock_account(&mut self) {
        self.locked_until = None;
        self.failed_login_attempts = 0;
        self.updated_at = Utc::now();
    }

    // ============================================
    // BUSINESS LOGIC - ACCOUNT STATUS
    // ============================================

    /// Suspend account
    pub fn suspend(&mut self) -> Result<(), AuthCredentialError> {
        if self.account_status == AccountStatus::Deleted {
            return Err(AuthCredentialError::AccountDeleted);
        }

        self.account_status = AccountStatus::Suspended;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Activate account
    pub fn activate(&mut self) -> Result<(), AuthCredentialError> {
        if self.account_status == AccountStatus::Deleted {
            return Err(AuthCredentialError::AccountDeleted);
        }

        self.account_status = AccountStatus::Active;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Soft delete account
    pub fn delete(&mut self) {
        self.account_status = AccountStatus::Deleted;
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Check if account is active
    pub fn is_active(&self) -> bool {
        self.account_status == AccountStatus::Active && !self.is_deleted()
    }

    /// Ensure account is active (for business operations)
    pub fn ensure_active(&self) -> Result<(), AuthCredentialError> {
        if self.is_deleted() {
            return Err(AuthCredentialError::AccountDeleted);
        }

        if self.account_status == AccountStatus::Suspended {
            return Err(AuthCredentialError::AccountSuspended);
        }

        if self.account_status != AccountStatus::Active {
            return Err(AuthCredentialError::AccountInactive);
        }

        Ok(())
    }

    /// Ensure account is not locked
    pub fn ensure_not_locked(&self) -> Result<(), AuthCredentialError> {
        if self.is_locked() {
            return Err(AuthCredentialError::AccountLocked {
                locked_until: self.locked_until,
            });
        }
        Ok(())
    }

    // ============================================
    // BUSINESS LOGIC - ROLE MANAGEMENT
    // ============================================

    /// Promote user to Admin role
    pub fn promote_to_admin(&mut self) {
        self.system_role = SystemRole::Admin;
        self.updated_at = Utc::now();
    }

    /// Promote user to Moderator role
    pub fn promote_to_moderator(&mut self) {
        self.system_role = SystemRole::Moderator;
        self.updated_at = Utc::now();
    }

    /// Demote user to regular User role
    pub fn demote_to_user(&mut self) {
        self.system_role = SystemRole::User;
        self.updated_at = Utc::now();
    }

    // ============================================
    // BUSINESS LOGIC - EMAIL CHANGE
    // ============================================

    /// Change email (requires re-verification)
    pub fn change_email(&mut self, new_email: Email) {
        self.email = new_email;
        self.email_verified = false;
        self.email_verification_token = None;
        self.email_verification_expires_at = None;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_id() -> Uuid {
        Uuid::new_v4()
    }

    fn create_test_email() -> Email {
        Email::new("test@example.com").unwrap()
    }

    fn create_test_password() -> PasswordHash {
        PasswordHash::from_hash("$argon2id$v=19$m=19456,t=2,p=1$...")
    }

    #[test]
    fn test_create_new_credential() {
        let email = create_test_email();
        let password = create_test_password();

        let credential = AuthCredential::new(create_test_user_id(), email.clone(), password);

        assert_eq!(credential.email(), &email);
        assert!(!credential.is_email_verified());
        assert_eq!(credential.failed_login_attempts(), 0);
        assert_eq!(credential.account_status(), &AccountStatus::Active);
        assert!(credential.is_active());
    }

    #[test]
    fn test_email_verification_flow() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        // Generate token
        let token = credential.generate_verification_token();
        assert!(credential.email_verification_token().is_some());
        assert!(credential.email_verification_expires_at().is_some());

        // Verify email
        credential.verify_email(&token).unwrap();
        assert!(credential.is_email_verified());
        assert!(credential.email_verification_token().is_none());
    }

    #[test]
    fn test_failed_login_attempts() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        // First 4 attempts should not lock
        for _ in 0..4 {
            let locked = credential.record_failed_login();
            assert!(!locked);
            assert!(!credential.is_locked());
        }

        // 5th attempt should lock
        let locked = credential.record_failed_login();
        assert!(locked);
        assert!(credential.is_locked());
        assert!(credential.locked_until().is_some());
    }

    #[test]
    fn test_successful_login_resets_attempts() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        // Record failed attempts
        credential.record_failed_login();
        credential.record_failed_login();
        assert_eq!(credential.failed_login_attempts(), 2);

        // Successful login should reset
        credential.record_successful_login(Some("192.168.1.1".to_string()));
        assert_eq!(credential.failed_login_attempts(), 0);
        assert!(!credential.is_locked());
        assert!(credential.last_login_at().is_some());
    }

    #[test]
    fn test_password_reset_flow() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        // Generate reset token
        let token = credential.generate_password_reset_token();
        assert!(credential.password_reset_token().is_some());

        // Reset password
        let new_password = PasswordHash::from_hash("new_hash");
        credential.reset_password(&token, new_password).unwrap();

        assert!(credential.password_reset_token().is_none());
        assert!(credential.password_changed_at().is_some());
    }

    #[test]
    fn test_account_suspension() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        assert!(credential.is_active());

        // Suspend account
        credential.suspend().unwrap();
        assert_eq!(credential.account_status(), &AccountStatus::Suspended);
        assert!(!credential.is_active());

        // Activate again
        credential.activate().unwrap();
        assert_eq!(credential.account_status(), &AccountStatus::Active);
        assert!(credential.is_active());
    }

    #[test]
    fn test_soft_delete() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        credential.delete();
        assert!(credential.is_deleted());
        assert_eq!(credential.account_status(), &AccountStatus::Deleted);
        assert!(!credential.is_active());
    }

    #[test]
    fn test_email_change_requires_reverification() {
        let mut credential = AuthCredential::new(
            create_test_user_id(),
            create_test_email(),
            create_test_password(),
        );

        // Verify email first
        let token = credential.generate_verification_token();
        credential.verify_email(&token).unwrap();
        assert!(credential.is_email_verified());

        // Change email
        let new_email = Email::new("new@example.com").unwrap();
        credential.change_email(new_email);

        // Should require re-verification
        assert!(!credential.is_email_verified());
        assert!(credential.email_verification_token().is_none());
    }
}
