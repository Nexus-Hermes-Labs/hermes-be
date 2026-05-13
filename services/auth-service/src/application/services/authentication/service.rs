use super::error::AuthApplicationError;
use super::user_profile_client::UserProfileClient;
use crate::application::events::UserCreatedEvent;
use crate::application::ports::unit_of_work::AuthUnitOfWorkFactory;
use crate::domain::auth_credential::EmailService;
use crate::domain::auth_credential::{
    AuthCredential, AuthCredentialRepository, Email, PasswordService,
};
use crate::domain::auth_session::{AuthSession, AuthSessionRepository, TokenHasher};
use chrono::Utc;
use crate::presentation::http::dto::{
    AuthResponse, ClientInfo, LoginRequest, LogoutResponse, RefreshTokenRequest, RegisterRequest,
    RegisterResponse, UserProfile,
};
use common::domain::event::IntoEventEnvelope;
use common::infrastructure::messaging::{EventPublisher, EventPublisherExt};
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
    event_publisher: Arc<dyn EventPublisher>,
    jwt_manager: Arc<JwtManager>,
    user_profile_client: Arc<dyn UserProfileClient>,
    email_service: Arc<dyn EmailService>,
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
        event_publisher: Arc<dyn EventPublisher>,
        jwt_manager: Arc<JwtManager>,
        user_profile_client: Arc<dyn UserProfileClient>,
        email_service: Arc<dyn EmailService>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            credential_repo,
            session_repo,
            uow_factory,
            password_service,
            token_hasher,
            event_publisher,
            jwt_manager,
            user_profile_client,
            email_service,
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
    /// 4. Call user-service via gRPC to create profile (SYNC)
    /// 5. Generate verification token and send email
    /// 6. Generate JWT tokens
    /// 7. Create session
    /// 8. Publish user.created event (ASYNC, non-blocking)
    /// 9. Return tokens + user profile
    pub async fn register(
        &self,
        request: RegisterRequest,
        client_info: ClientInfo,
    ) -> Result<RegisterResponse, AuthApplicationError> {
        info!(email = %request.email, username = %request.username, "Registration started");

        // ═══════════════════════════════════════════════════
        // STEP 1: Validate Email
        // ═══════════════════════════════════════════════════
        let email = Email::new(&request.email)
            .map_err(|_| AuthApplicationError::InvalidEmail(request.email.clone()))?;

        if self.credential_repo.exists_by_email(&email).await? {
            warn!(email = %request.email, "Registration failed: email already exists");
            return Err(AuthApplicationError::EmailAlreadyExists(request.email));
        }

        // ═══════════════════════════════════════════════════
        // STEP 2: Hash Password
        // ═══════════════════════════════════════════════════
        let password_hash = self
            .password_service
            .hash_password(&request.password)
            .map_err(|_e| AuthApplicationError::HashingFailed)?;

        // ═══════════════════════════════════════════════════
        // STEP 3: Create User Profile via gRPC
        // ═══════════════════════════════════════════════════
        let profile_info = self
            .user_profile_client
            .create_profile(
                request.username.to_owned(),
                request.display_name.to_owned(),
                email.as_str().to_owned(),
            )
            .await?;

        // ═══════════════════════════════════════════════════
        // STEP 4: Create Auth Credential using obtained user_id
        // ═══════════════════════════════════════════════════
        let credential = AuthCredential::new(profile_info.user_id, email.clone(), password_hash);

        info!(
            credential_id = %credential.id(),
            user_id = %credential.user_id(),
            email = %credential.email(),
            "Auth credential created"
        );

        let user_profile = UserProfile {
            user_id: profile_info.user_id,
            email: credential.email().as_str().to_string(),
            username: profile_info.username,
            display_name: profile_info.display_name,
            avatar: profile_info.avatar_url,
            bio: profile_info.bio,
            created_at: profile_info.created_at,
        };

        info!(
            user_id = %credential.id(),
            username = %user_profile.username,
            "User profile created via user-service"
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

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        uow.credentials().save(&credential).await.map_err(|e| {
            error!(error = %e, "STEP 7 FAILED: credential save");
            AuthApplicationError::RepositoryError(e.to_string())
        })?;

        uow.credentials()
            .set_verification_token(
                credential.id(),
                &verification_token,
                VERIFICATION_TOKEN_EXPIRY_HOURS,
            )
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        uow.sessions().save(&session).await.map_err(|e| {
            error!(error = %e, "STEP 7 FAILED: session save");
            AuthApplicationError::RepositoryError(e.to_string())
        })?;

        uow.commit()
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

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
        // STEP 8: Publish Event (ASYNC, Non-blocking)
        // ═══════════════════════════════════════════════════
        let event = UserCreatedEvent::new(
            credential.user_id(),
            credential.email().as_str().to_string(),
            request.username.clone(),
            request.display_name.clone(),
        );

        // Fire and forget - don't fail registration if event fails
        if let Err(e) = self
            .event_publisher
            .publish(
                "user_profile.created",
                &IntoEventEnvelope::into_envelope(event, &self.service_name),
            )
            .await
        {
            error!(
                error = %e,
                user_id = %credential.id(),
                "Failed to publish user_profile.created event (non-critical)"
            );
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

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        uow.credentials()
            .update(&credential)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        // Enforce one active session per device: if the same client (identified
        // by `device_id`) already has an active session, revoke it before
        // inserting the new one. Without a device_id, prior behavior is kept
        // (multiple concurrent sessions allowed).
        if let Some(ref did) = device_id {
            let revoked = uow
                .sessions()
                .revoke_active_by_device(credential.id(), did)
                .await
                .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;
            if revoked > 0 {
                info!(
                    user_id = %credential.id(),
                    device_id = %did,
                    revoked_count = revoked,
                    "Replaced existing session(s) for same device"
                );
            }
        }

        uow.sessions()
            .save(&session)
            .await
            .map_err(|e| AuthApplicationError::RepositoryError(e.to_string()))?;

        uow.commit()
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
}
