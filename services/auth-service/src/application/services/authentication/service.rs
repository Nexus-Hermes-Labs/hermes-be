use super::error::AuthApplicationError;
use crate::application::events::UserCreatedEvent;
use crate::application::ports::unit_of_work::AuthUnitOfWorkFactory;
use crate::domain::auth_credential::EmailService;
use crate::domain::auth_credential::{
    AuthCredential, AuthCredentialRepository, Email, PasswordService,
};
use crate::domain::auth_session::{AuthSession, AuthSessionRepository, TokenHasher};
use crate::domain::password_history::{PasswordHistory, PasswordHistoryRepository};
use crate::domain::password_policy::PasswordPolicy;
use crate::domain::rate_limit::{RateLimitResult, RateLimiter};
use crate::presentation::http::dto::{
    AuthResponse, ChangePasswordRequest, ClientInfo, ForgotPasswordRequest, ForgotPasswordResponse,
    LoginRequest, LogoutResponse, RefreshTokenRequest, RegisterRequest, RegisterResponse,
    ResetPasswordRequest, ResetPasswordResponse, UserProfile,
};
use chrono::Utc;
use common::domain::event::IntoEventEnvelope;
use common::infrastructure::outbox::{AggregateType, NewOutboxEvent, OutboxEventType};
use common::infrastructure::security::jwt_manager::JwtManager;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ============================================
// CONFIGURATION
// ============================================

const ACCESS_TOKEN_EXPIRY_HOURS: i64 = 6;
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;
const VERIFICATION_TOKEN_EXPIRY_HOURS: i64 = 24;

fn map_registration_repository_error(
    error: common::infrastructure::persistence::error::RepositoryError,
    email: &str,
) -> AuthApplicationError {
    if is_unique_violation(&error) {
        return AuthApplicationError::EmailAlreadyExists(email.to_string());
    }
    AuthApplicationError::RepositoryError(error.to_string())
}

fn is_unique_violation(
    error: &common::infrastructure::persistence::error::RepositoryError,
) -> bool {
    match error {
        common::infrastructure::persistence::error::RepositoryError::Database(
            sqlx::Error::Database(db_error),
        ) => {
            db_error.code().as_deref() == Some("23505")
                && db_error.constraint() == Some("auth_credentials_email_key")
        }
        common::infrastructure::persistence::error::RepositoryError::DuplicateEntry(_) => true,
        _ => false,
    }
}

// ============================================
// AUTHENTICATION SERVICE
/// Main authentication service
///
/// Orchestrates authentication workflows using:
/// - Domain entities (AuthCredential, AuthSession)
/// - Repositories (AuthCredentialRepository, AuthSessionRepository)
/// - Domain services (PasswordService, EmailService)
/// - Infrastructure services (JwtManager, EventPublisher, UserProfileClient)
pub struct AuthService {
    service_name: String,
    credential_repo: Arc<dyn AuthCredentialRepository>,
    session_repo: Arc<dyn AuthSessionRepository>,
    uow_factory: Arc<dyn AuthUnitOfWorkFactory>,
    password_service: Arc<dyn PasswordService>,
    token_hasher: Arc<dyn TokenHasher>,
    jwt_manager: Arc<JwtManager>,
    email_service: Arc<dyn EmailService>,
    password_history_repo: Arc<dyn PasswordHistoryRepository>,
    password_policy: PasswordPolicy,
    rate_limiter: Arc<dyn RateLimiter>,
}

// ============================================

impl AuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service_name: impl Into<String>,
        credential_repo: Arc<dyn AuthCredentialRepository>,
        session_repo: Arc<dyn AuthSessionRepository>,
        uow_factory: Arc<dyn AuthUnitOfWorkFactory>,
        password_service: Arc<dyn PasswordService>,
        token_hasher: Arc<dyn TokenHasher>,
        jwt_manager: Arc<JwtManager>,
        email_service: Arc<dyn EmailService>,
        password_history_repo: Arc<dyn PasswordHistoryRepository>,
        password_policy: PasswordPolicy,
        rate_limiter: Arc<dyn RateLimiter>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            credential_repo,
            session_repo,
            uow_factory,
            password_service,
            token_hasher,
            jwt_manager,
            email_service,
            password_history_repo,
            password_policy,
            rate_limiter,
        }
    }

    fn generate_verification_token(&self) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    // ============================================
    // REGISTRATION
    // ============================================

    /// Register a new user
    ///
    /// Workflow:
    /// 1. Validate email not already registered
    /// 2. Hash password
    /// 3. Create auth credential
    /// 4. Write user.created outbox event for user-service
    /// 5. Generate verification token
    /// 6. Generate JWT tokens
    /// 7. Create session
    /// 8. Commit transaction, then send email
    /// 9. Return tokens + eventual user profile
    pub async fn register(
        &self,
        request: RegisterRequest,
        client_info: ClientInfo,
    ) -> Result<RegisterResponse, AuthApplicationError> {
        info!(email = %request.email, username = %request.username, "Registration started");

        // ═══════════════════════════════════════════════════
        // STEP 1: Validate Email
        // ═══════════════════════════════════════════════════
        // Uniqueness is enforced by the auth_credentials_email_key constraint;
        // a duplicate INSERT inside the UoW is mapped to EmailAlreadyExists.
        let email = Email::new(&request.email)
            .map_err(|_| AuthApplicationError::InvalidEmail(request.email.clone()))?;

        // ═══════════════════════════════════════════════════
        // STEP 1b: Validate Password Policy
        // ═══════════════════════════════════════════════════
        if let Err(violations) = self.password_policy.validate(&request.password) {
            return Err(AuthApplicationError::PasswordPolicyViolation(
                violations.join("; "),
            ));
        }

        // ═══════════════════════════════════════════════════
        // STEP 2: Hash Password
        // ═══════════════════════════════════════════════════
        let password_hash = self
            .password_service
            .hash_password(&request.password)
            .map_err(|_e| AuthApplicationError::HashingFailed)?;

        let user_id = Uuid::new_v4();
        let credential = AuthCredential::new(user_id, email.clone(), password_hash);

        info!(
            credential_id = %credential.id(),
            user_id = %credential.user_id(),
            email = %credential.email(),
            "Auth credential created"
        );

        // ═══════════════════════════════════════════════════
        // STEP 5: Generate verification token
        // ═══════════════════════════════════════════════════
        let verification_token = self.generate_verification_token();

        // ═══════════════════════════════════════════════════
        // STEP 6: Generate Tokens
        // ═══════════════════════════════════════════════════
        let access_token = self
            .jwt_manager
            .create_access_token(
                credential.user_id(),
                credential.email().as_str(),
                credential.system_role(),
                ACCESS_TOKEN_EXPIRY_HOURS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(
                credential.user_id(),
                credential.email().as_str(),
                REFRESH_TOKEN_EXPIRY_DAYS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        // ═══════════════════════════════════════════════════
        // STEP 7: Persist credential + verification token + session atomically
        // ═══════════════════════════════════════════════════
        let token_hash = self
            .token_hasher
            .hash(&refresh_token)
            .map_err(|_| AuthApplicationError::TokenGenerationFailed(refresh_token.clone()))?;
        let session = AuthSession::create(
            credential.id(),
            token_hash.as_str().to_owned(),
            REFRESH_TOKEN_EXPIRY_DAYS,
            client_info.ip_address,
            client_info.user_agent,
            client_info.device_id,
        );

        let user_profile = UserProfile {
            user_id,
            email: credential.email().as_str().to_string(),
            username: request.username.clone(),
            display_name: request.display_name.clone(),
            avatar: None,
            bio: None,
            created_at: Utc::now(),
        };

        info!(
            user_id = %credential.user_id(),
            username = %user_profile.username,
            "User profile creation queued via outbox"
        );

        let event = UserCreatedEvent::new(
            credential.user_id(),
            credential.email().as_str().to_string(),
            user_profile.username.clone(),
            user_profile.display_name.clone(),
        );

        let envelope = event.into_envelope(&self.service_name);
        let outbox_event = NewOutboxEvent {
            id: envelope.event_id,
            aggregate_id: envelope.aggregate_id,
            aggregate_type: AggregateType::User,
            event_type: OutboxEventType::UserCreated,
            payload: serde_json::to_value(&envelope)
                .map_err(|e| AuthApplicationError::Internal(e.to_string()))?,
        };

        let credential_for_tx = credential.clone();
        let session_for_tx = session.clone();
        let outbox_event_for_tx = outbox_event.clone();
        let verification_token_for_tx = verification_token.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.credentials().save(&credential_for_tx).await?;
                    uow.credentials()
                        .set_verification_token(
                            credential_for_tx.id(),
                            &verification_token_for_tx,
                            VERIFICATION_TOKEN_EXPIRY_HOURS,
                        )
                        .await?;
                    uow.sessions().save(&session_for_tx).await?;
                    uow.outbox().save(&outbox_event_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| map_registration_repository_error(e, &request.email))?;

        info!(
            user_id = %credential.id(),
            session_id = %session.id(),
            "Session created"
        );

        // Send verification email after successful commit (fire and forget)
        if let Err(e) = self
            .email_service
            .send_verification_email(credential.email().as_str(), &verification_token)
            .await
        {
            error!(error = %e, user_id = %credential.id(), "Failed to send verification email (non-critical)");
        }

        // ═══════════════════════════════════════════════════
        // STEP 9: Return Response
        // ═══════════════════════════════════════════════════
        info!(
            user_id = %credential.id(),
            username = %user_profile.username,
            "Registration completed successfully"
        );

        Ok(RegisterResponse {
            access_token,
            refresh_token,
            expires_in: ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60,
            token_type: "Bearer".to_string(),
            user: user_profile,
        })
    }

    // ============================================
    // LOGIN
    // ============================================

    /// Login with email and password
    ///
    /// Workflow:
    /// 1. Find credential by email
    /// 2. Check account status (locked/suspended/deleted/verified)
    /// 3. Verify password
    /// 4. Record login attempt
    /// 5. Generate tokens
    /// 6. Create session
    /// 7. Return tokens
    pub async fn login(
        &self,
        request: LoginRequest,
        client_info: ClientInfo,
    ) -> Result<AuthResponse, AuthApplicationError> {
        info!(email = %request.email, "Login attempt");

        // ═══════════════════════════════════════════════════
        // RATE LIMIT CHECK
        // ═══════════════════════════════════════════════════
        let rate_key = format!("login:{}", request.email.to_lowercase());
        match self.rate_limiter.check_rate_limit(&rate_key, 10, 300).await {
            Ok(RateLimitResult::Exceeded {
                retry_after_seconds,
            }) => {
                warn!(email = %request.email, "Login rate limited");
                return Err(AuthApplicationError::RateLimited {
                    retry_after_seconds,
                });
            }
            Err(e) => {
                warn!(error = %e, "Rate limiter error, proceeding without limit");
            }
            _ => {}
        }

        // ═══════════════════════════════════════════════════
        // STEP 1: Find Credential
        // ═══════════════════════════════════════════════════
        let email = Email::new(&request.email)
            .map_err(|e| AuthApplicationError::InvalidEmail(e.to_string()))?;

        let mut credential = self
            .credential_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| {
                warn!(email = %request.email, "Login failed: credential not found");
                AuthApplicationError::InvalidCredentials
            })?;

        // ═══════════════════════════════════════════════════
        // STEP 2: Check Account Status
        // ═══════════════════════════════════════════════════
        if !credential.is_email_verified() {
            warn!(user_id = %credential.id(), "Login failed: email not verified");
            return Err(AuthApplicationError::EmailNotVerified);
        }

        if credential.is_locked() {
            warn!(
                user_id = %credential.id(),
                locked_until = ?credential.locked_until(),
                "Login failed: account locked"
            );
            return Err(AuthApplicationError::AccountLocked {
                locked_until: credential.locked_until().unwrap_or_default(),
            });
        }

        if credential.account_status().is_suspended() {
            warn!(user_id = %credential.id(), "Login failed: account suspended");
            return Err(AuthApplicationError::AccountSuspended);
        }

        if credential.is_deleted() {
            warn!(user_id = %credential.id(), "Login failed: account deleted");
            return Err(AuthApplicationError::AccountDeleted);
        }

        // ═══════════════════════════════════════════════════
        // STEP 3: Verify Password
        // ═══════════════════════════════════════════════════
        let is_valid = self
            .password_service
            .verify_password(&request.password, credential.password_hash())
            .map_err(|_e| AuthApplicationError::WeakPassword)?;

        if !is_valid {
            warn!(user_id = %credential.id(), "Login failed: invalid password");

            // Record failed attempt
            let is_locked = credential.record_failed_login();
            self.credential_repo.update(&credential).await?;

            if is_locked {
                warn!(
                    user_id = %credential.id(),
                    "Account locked due to too many failed attempts"
                );
                return Err(AuthApplicationError::AccountLocked {
                    locked_until: credential.locked_until().unwrap_or_default(),
                });
            }

            return Err(AuthApplicationError::InvalidCredentials);
        }

        // ═══════════════════════════════════════════════════
        // STEP 4: Record Successful Login
        // ═══════════════════════════════════════════════════
        credential.record_successful_login(client_info.ip_address.clone());

        info!(user_id = %credential.id(), "Login successful");

        // ═══════════════════════════════════════════════════
        // STEP 5: Generate Tokens
        // ═══════════════════════════════════════════════════
        let access_token = self
            .jwt_manager
            .create_access_token(
                credential.user_id(),
                credential.email().as_str(),
                credential.system_role(),
                ACCESS_TOKEN_EXPIRY_HOURS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(
                credential.user_id(),
                credential.email().as_str(),
                REFRESH_TOKEN_EXPIRY_DAYS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        // ═══════════════════════════════════════════════════
        // STEP 6: Persist login record + new session atomically
        // ═══════════════════════════════════════════════════
        let token_hash = self
            .token_hasher
            .hash(&refresh_token)
            .map_err(|_| AuthApplicationError::TokenGenerationFailed(refresh_token.to_string()))?;
        let device_id = client_info.device_id.clone();
        let session = AuthSession::create(
            credential.id(),
            token_hash.as_str().to_owned(),
            REFRESH_TOKEN_EXPIRY_DAYS,
            client_info.ip_address,
            client_info.user_agent,
            device_id.clone(),
        );

        // Enforce one active session per device: if the same client (identified
        // by `device_id`) already has an active session, revoke it before
        // inserting the new one. Without a device_id, prior behavior is kept
        // (multiple concurrent sessions allowed).
        let credential_for_tx = credential.clone();
        let session_for_tx = session.clone();
        let device_id_for_tx = device_id.clone();
        let credential_id = credential.id();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.credentials().update(&credential_for_tx).await?;

                    if let Some(did) = device_id_for_tx.as_deref() {
                        let revoked = uow
                            .sessions()
                            .revoke_active_by_device(credential_id, did)
                            .await?;
                        if revoked > 0 {
                            info!(
                                user_id = %credential_id,
                                device_id = %did,
                                revoked_count = revoked,
                                "Replaced existing session(s) for same device"
                            );
                        }
                    }

                    uow.sessions().save(&session_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        Ok(AuthResponse::new(
            access_token,
            refresh_token,
            ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60,
        ))
    }

    // ============================================
    // EMAIL VERIFICATION
    // ============================================

    /// Verify user's email address
    pub async fn verify_email(&self, token: &str) -> Result<(), AuthApplicationError> {
        info!(token = %token, "Email verification attempt");

        let mut credential = self
            .credential_repo
            .find_by_verification_token(token)
            .await?
            .ok_or(AuthApplicationError::InvalidToken)?;

        credential
            .verify_email(token)
            .map_err(|_| AuthApplicationError::InvalidToken)?;

        self.credential_repo.update(&credential).await?;

        info!(user_id = %credential.user_id(), "Email verified successfully");

        Ok(())
    }

    // ============================================
    // REFRESH TOKEN
    // ============================================

    /// Refresh access token using refresh token.
    ///
    /// On every successful refresh the refresh token is **rotated**: a fresh
    /// JWT is issued and the session row's hash is updated. The previous hash
    /// is kept on the row for a short grace window so retries surviving lost
    /// responses can be served idempotently.
    ///
    /// If a refresh token whose hash matches the *previous* slot is presented
    /// outside of the grace window, we treat it as token theft and revoke
    /// every session belonging to the credential.
    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
        client_info: ClientInfo,
    ) -> Result<AuthResponse, AuthApplicationError> {
        debug!("Refreshing access token");

        let claims = self
            .jwt_manager
            .verify_refresh_token(&request.refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token");
                AuthApplicationError::InvalidToken
            })?;

        let user_id = claims.user_id().map_err(|e| {
            error!(error = %e, "Invalid user_id in token claims");
            AuthApplicationError::InvalidToken
        })?;

        let credential = self
            .credential_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                warn!("Refresh failed: credential not found");
                AuthApplicationError::InvalidToken
            })?;

        let incoming_hash = self
            .token_hasher
            .hash(&request.refresh_token)
            .map_err(|_| AuthApplicationError::HashingFailed)?;

        // Try the active hash first.
        if let Some(mut session) = self
            .session_repo
            .find_by_refresh_token_hash(incoming_hash.as_str())
            .await?
        {
            if session.credential_id() != credential.id() {
                warn!("Refresh failed: session does not belong to user");
                return Err(AuthApplicationError::InvalidToken);
            }
            if !session.is_valid() {
                warn!(session_id = %session.id(), "Refresh failed: session invalid");
                return Err(AuthApplicationError::InvalidToken);
            }

            return self
                .rotate_and_respond(
                    &mut session,
                    user_id,
                    &claims.email,
                    claims.role,
                    &client_info,
                )
                .await;
        }

        // No active match. Maybe the caller is presenting the previous hash —
        // either an in-flight retry (grace window) or a leaked stale token.
        if let Some(mut session) = self
            .session_repo
            .find_by_previous_token_hash(incoming_hash.as_str())
            .await?
        {
            if session.credential_id() != credential.id() {
                warn!("Refresh failed: previous-hash session does not belong to user");
                return Err(AuthApplicationError::InvalidToken);
            }

            if session.is_in_grace_window(Utc::now()) && !session.is_revoked() {
                debug!(
                    session_id = %session.id(),
                    "Previous refresh token presented inside grace window; rotating again"
                );
                return self
                    .rotate_and_respond(
                        &mut session,
                        user_id,
                        &claims.email,
                        claims.role,
                        &client_info,
                    )
                    .await;
            }

            // Out of grace (or already revoked) → token theft suspected.
            warn!(
                user_id = %user_id,
                session_id = %session.id(),
                "Refresh token reuse detected; revoking all sessions for credential"
            );
            self.session_repo
                .revoke_all_by_credential_id(credential.id())
                .await?;
            return Err(AuthApplicationError::InvalidToken);
        }

        warn!("Refresh failed: token hash not recognized");
        Err(AuthApplicationError::InvalidToken)
    }

    /// Mint new access + refresh tokens, rotate the session in place,
    /// persist, and return the response.
    async fn rotate_and_respond(
        &self,
        session: &mut AuthSession,
        user_id: Uuid,
        email: &str,
        role: common::infrastructure::security::jwt_manager::SystemRole,
        client_info: &ClientInfo,
    ) -> Result<AuthResponse, AuthApplicationError> {
        let access_token = self
            .jwt_manager
            .create_access_token(user_id, email, role, ACCESS_TOKEN_EXPIRY_HOURS)
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let new_refresh_token = self
            .jwt_manager
            .create_refresh_token(user_id, email, REFRESH_TOKEN_EXPIRY_DAYS)
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let new_hash = self
            .token_hasher
            .hash(&new_refresh_token)
            .map_err(|_| AuthApplicationError::HashingFailed)?;

        session.rotate(
            new_hash.as_str().to_owned(),
            REFRESH_TOKEN_EXPIRY_DAYS,
            client_info.ip_address.clone(),
        );
        self.session_repo.update(session).await?;

        debug!(session_id = %session.id(), "Refresh token rotated");

        Ok(AuthResponse::new(
            access_token,
            new_refresh_token,
            ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60,
        ))
    }

    // ============================================
    // LOGOUT
    // ============================================

    /// Logout (revoke sessions)
    pub async fn logout(
        &self,
        user_id: Uuid,
        session_id: Option<Uuid>,
        all_devices: bool,
    ) -> Result<LogoutResponse, AuthApplicationError> {
        info!(user_id = %user_id, all_devices = all_devices, "Logout");

        let credential = self
            .credential_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(AuthApplicationError::UserNotFound(user_id))?;

        let revoked_count = if all_devices {
            // Revoke all sessions
            self.session_repo
                .revoke_all_by_credential_id(credential.id())
                .await?
        } else if let Some(session_id) = session_id {
            // Revoke specific session
            if let Some(mut session) = self.session_repo.find_by_id(session_id).await? {
                session.revoke();
                self.session_repo.update(&session).await?;
                1
            } else {
                0
            }
        } else {
            0
        };

        info!(
            user_id = %user_id,
            sessions_revoked = revoked_count,
            "Logout successful"
        );

        Ok(LogoutResponse {
            message: "Logged out successfully".to_string(),
            sessions_revoked: revoked_count,
        })
    }

    // ============================================
    // FORGOT PASSWORD
    // ============================================

    pub async fn forgot_password(
        &self,
        request: ForgotPasswordRequest,
    ) -> Result<ForgotPasswordResponse, AuthApplicationError> {
        info!(email = %request.email, "Forgot password request");

        let rate_key = format!("forgot_password:{}", request.email.to_lowercase());
        match self.rate_limiter.check_rate_limit(&rate_key, 3, 600).await {
            Ok(RateLimitResult::Exceeded {
                retry_after_seconds,
            }) => {
                warn!(email = %request.email, "Forgot password rate limited");
                return Err(AuthApplicationError::RateLimited {
                    retry_after_seconds,
                });
            }
            Err(e) => {
                warn!(error = %e, "Rate limiter error, proceeding without limit");
            }
            _ => {}
        }

        let email = Email::new(&request.email)
            .map_err(|_| AuthApplicationError::InvalidEmail(request.email.clone()))?;

        // Always return success to prevent email enumeration
        let credential = self.credential_repo.find_by_email(&email).await?;

        if let Some(mut credential) = credential {
            let token = credential.generate_password_reset_token();
            self.credential_repo.update(&credential).await?;

            if let Err(e) = self
                .email_service
                .send_password_reset_email(credential.email().as_str(), &token)
                .await
            {
                error!(error = %e, "Failed to send password reset email (non-critical)");
            }
        }

        Ok(ForgotPasswordResponse {
            message: "If the email exists, a password reset link has been sent.".to_string(),
        })
    }

    // ============================================
    // RESET PASSWORD (via token)
    // ============================================

    pub async fn reset_password(
        &self,
        request: ResetPasswordRequest,
    ) -> Result<ResetPasswordResponse, AuthApplicationError> {
        info!("Password reset attempt");

        if let Err(violations) = self.password_policy.validate(&request.new_password) {
            return Err(AuthApplicationError::PasswordPolicyViolation(
                violations.join("; "),
            ));
        }

        let mut credential = self
            .credential_repo
            .find_by_password_reset_token(&request.token)
            .await?
            .ok_or(AuthApplicationError::InvalidToken)?;

        // Check password history
        let history = self
            .password_history_repo
            .find_recent_by_credential(credential.id(), self.password_policy.history_count as i64)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        let new_hash = self
            .password_service
            .hash_password(&request.new_password)
            .map_err(|_| AuthApplicationError::HashingFailed)?;

        for entry in &history {
            if self
                .password_service
                .verify_password(
                    &request.new_password,
                    &crate::domain::auth_credential::PasswordHash::from_hash(entry.password_hash()),
                )
                .unwrap_or(false)
            {
                return Err(AuthApplicationError::PasswordRecentlyUsed);
            }
        }

        // Save current password to history before changing
        let history_entry = PasswordHistory::new(
            credential.id(),
            credential.password_hash().as_str().to_string(),
        );
        self.password_history_repo
            .save(&history_entry)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        credential
            .reset_password(&request.token, new_hash)
            .map_err(|_| AuthApplicationError::InvalidToken)?;

        self.credential_repo.update(&credential).await?;

        // Prune old history entries
        let _ = self
            .password_history_repo
            .delete_oldest_beyond_limit(credential.id(), self.password_policy.history_count as i64)
            .await;

        // Revoke all sessions on password reset
        self.session_repo
            .revoke_all_by_credential_id(credential.id())
            .await?;

        info!(user_id = %credential.user_id(), "Password reset successfully");

        Ok(ResetPasswordResponse {
            message: "Password has been reset successfully. Please log in again.".to_string(),
        })
    }

    // ============================================
    // CHANGE PASSWORD (authenticated)
    // ============================================

    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<ResetPasswordResponse, AuthApplicationError> {
        info!(user_id = %user_id, "Change password request");

        if let Err(violations) = self.password_policy.validate(&request.new_password) {
            return Err(AuthApplicationError::PasswordPolicyViolation(
                violations.join("; "),
            ));
        }

        let mut credential = self
            .credential_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(AuthApplicationError::UserNotFound(user_id))?;

        // Verify current password
        let is_valid = self
            .password_service
            .verify_password(&request.current_password, credential.password_hash())
            .map_err(|_| AuthApplicationError::HashingFailed)?;

        if !is_valid {
            return Err(AuthApplicationError::InvalidCredentials);
        }

        // Check password history
        let history = self
            .password_history_repo
            .find_recent_by_credential(credential.id(), self.password_policy.history_count as i64)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        for entry in &history {
            if self
                .password_service
                .verify_password(
                    &request.new_password,
                    &crate::domain::auth_credential::PasswordHash::from_hash(entry.password_hash()),
                )
                .unwrap_or(false)
            {
                return Err(AuthApplicationError::PasswordRecentlyUsed);
            }
        }

        // Also check against current password
        if self
            .password_service
            .verify_password(&request.new_password, credential.password_hash())
            .unwrap_or(false)
        {
            return Err(AuthApplicationError::PasswordRecentlyUsed);
        }

        // Save current password to history
        let history_entry = PasswordHistory::new(
            credential.id(),
            credential.password_hash().as_str().to_string(),
        );
        self.password_history_repo
            .save(&history_entry)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        let new_hash = self
            .password_service
            .hash_password(&request.new_password)
            .map_err(|_| AuthApplicationError::HashingFailed)?;

        credential.change_password(new_hash);
        self.credential_repo.update(&credential).await?;

        // Prune old history
        let _ = self
            .password_history_repo
            .delete_oldest_beyond_limit(credential.id(), self.password_policy.history_count as i64)
            .await;

        info!(user_id = %user_id, "Password changed successfully");

        Ok(ResetPasswordResponse {
            message: "Password changed successfully.".to_string(),
        })
    }
}
