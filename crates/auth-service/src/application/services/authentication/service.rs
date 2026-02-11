use super::error::AuthApplicationError;
use crate::application::events::UserCreatedEvent;
use crate::domain::auth_credential::{
    AuthCredential, AuthCredentialRepository, Email, PasswordHash, PasswordService,
};
use crate::domain::auth_session::{AuthSession, AuthSessionRepository, TokenHasher};
use crate::presentation::http::dto::{AuthResponse, ClientInfo, LoginRequest, LogoutResponse, RefreshTokenRequest, RegisterRequest, RegisterResponse, UserProfile};
use common::domain::event::IntoEventEnvelope;
use common::infrastructure::messaging::EventPublisher;
use common::infrastructure::security::jwt_manager::JwtManager;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ============================================
// CONFIGURATION
// ============================================

const ACCESS_TOKEN_EXPIRY_HOURS: i64 = 6;
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;
const MAX_FAILED_LOGIN_ATTEMPTS: i32 = 5;

// ============================================
// AUTHENTICATION SERVICE
/// Main authentication service
///
/// Orchestrates authentication workflows using:
/// - Domain entities (AuthCredential, AuthSession)
/// - Repositories (AuthCredentialRepository, AuthSessionRepository)
/// - Domain services (PasswordService)
/// - Infrastructure services (JwtManager, EventPublisher)
pub struct AuthService<CR, SR, PS, TH, EP>
where
    CR: AuthCredentialRepository,
    SR: AuthSessionRepository,
    PS: PasswordService,
    TH: TokenHasher,
    EP: EventPublisher,
{
    service_name: String,
    credential_repo: Arc<CR>,
    session_repo: Arc<SR>,
    password_service: Arc<PS>,
    token_hasher: Arc<TH>,
    event_publisher: Arc<EP>,
    jwt_manager: Arc<JwtManager>,
}

// ============================================

impl<CR, SR, PS, TH, EP> AuthService<CR, SR, PS, TH, EP>
where
    CR: AuthCredentialRepository,
    SR: AuthSessionRepository,
    PS: PasswordService,
    TH: TokenHasher,
    EP: EventPublisher,
{
    pub fn new(
        service_name: impl Into<String>,
        credential_repo: Arc<CR>,
        session_repo: Arc<SR>,
        password_service: Arc<PS>,
        token_hasher: Arc<TH>,
        event_publisher: Arc<EP>,
        jwt_manager: Arc<JwtManager>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            credential_repo,
            session_repo,
            password_service,
            token_hasher,
            event_publisher,
            jwt_manager,
        }
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
    /// 5. Generate JWT tokens
    /// 6. Create session
    /// 7. Publish user.created event (ASYNC, non-blocking)
    /// 8. Return tokens + user profile
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
            .map_err(|e| AuthApplicationError::HashingFailed)?;

        // ═══════════════════════════════════════════════════
        // STEP 3: Create Auth Credential
        // ═══════════════════════════════════════════════════
        let credential = AuthCredential::new(email.clone(), password_hash);
        self.credential_repo.save(&credential).await?;

        info!(
            user_id = %credential.id(),
            email = %credential.email(),
            "Auth credential created"
        );

        // ═══════════════════════════════════════════════════
        // STEP 4: Create User Profile via gRPC (SYNC)
        // ═══════════════════════════════════════════════════
        // TODO: Implement gRPC call to user-service
        // let user_profile = self.user_service_client
        //     .create_user_profile(CreateUserProfileRequest {
        //         user_id: credential.id().to_string(),
        //         username: request.username.clone(),
        //         display_name: request.display_name.clone(),
        //     })
        //     .await
        //     .map_err(|e| {
        //         error!(error = %e, user_id = %credential.id(), "gRPC call failed");
        //
        //         // ROLLBACK: Delete credential
        //         if let Err(delete_err) = self.credential_repo.delete(credential.id()).await {
        //             error!(error = %delete_err, "Failed to rollback credential");
        //         }
        //
        //         AuthApplicationError::UserProfileCreationFailed(e.to_string())
        //     })?;

        // TEMPORARY: Mock user profile until gRPC is implemented
        let user_profile = UserProfile {
            user_id: credential.id(),
            email: credential.email().as_str().to_string(),
            username: request.username.clone(),
            discriminator: "0001".to_string(), // TODO: From user-service
            display_name: request.display_name.clone(),
            avatar: None,
            bio: None,
            created_at: credential.created_at(),
        };

        info!(
            user_id = %credential.id(),
            username = %user_profile.username,
            "User profile created (mock)"
        );

        // ═══════════════════════════════════════════════════
        // STEP 5: Generate Tokens
        // ═══════════════════════════════════════════════════
        let access_token = self
            .jwt_manager
            .create_access_token(
                credential.id(),
                credential.email().as_str(),
                ACCESS_TOKEN_EXPIRY_HOURS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(
                credential.id(),
                credential.email().as_str(),
                REFRESH_TOKEN_EXPIRY_DAYS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        // ═══════════════════════════════════════════════════
        // STEP 6: Create Session
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
        );

        self.session_repo.save(&session).await?;

        info!(
            user_id = %credential.id(),
            session_id = %session.id(),
            "Session created"
        );

        // ═══════════════════════════════════════════════════
        // STEP 7: Publish Event (ASYNC, Non-blocking)
        // ═══════════════════════════════════════════════════
        let event = UserCreatedEvent::new(
            credential.id(),
            credential.email().as_str().to_string(),
            request.username.clone(),
            request.display_name.clone(),
        );

        // Fire and forget - don't fail registration if event fails
        if let Err(e) = self
            .event_publisher
            .publish(
                "user.created",
                &IntoEventEnvelope::into_envelope(event, &self.service_name),
            )
            .await
        {
            error!(
                error = %e,
                user_id = %credential.id(),
                "Failed to publish user.created event (non-critical)"
            );
        }

        // ═══════════════════════════════════════════════════
        // STEP 8: Return Response
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
    /// 2. Check account status (locked/suspended/deleted)
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
            //TODO: map_err weak pass degis.
            .map_err(|e| AuthApplicationError::WeakPassword)?;

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
        self.credential_repo.update(&credential).await?;

        info!(user_id = %credential.id(), "Login successful");

        // ═══════════════════════════════════════════════════
        // STEP 5: Generate Tokens (WITHOUT ROLE)
        // ═══════════════════════════════════════════════════
        let access_token = self
            .jwt_manager
            .create_access_token(
                credential.id(),
                credential.email().as_str(),
                ACCESS_TOKEN_EXPIRY_HOURS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(
                credential.id(),
                credential.email().as_str(),
                REFRESH_TOKEN_EXPIRY_DAYS,
            )
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        // ═══════════════════════════════════════════════════
        // STEP 6: Create Session
        // ═══════════════════════════════════════════════════
        let token_hash = self
            .token_hasher
            .hash(&refresh_token)
            .map_err(|_| AuthApplicationError::TokenGenerationFailed(refresh_token.to_string()))?;
        let session = AuthSession::create(
            credential.id(),
            token_hash.as_str().to_owned(),
            REFRESH_TOKEN_EXPIRY_DAYS,
            client_info.ip_address,
            client_info.user_agent,
        );

        self.session_repo.save(&session).await?;

        Ok(AuthResponse::new(
            access_token,
            refresh_token,
            ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60,
        ))
    }

    // ============================================
    // REFRESH TOKEN
    // ============================================

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<AuthResponse, AuthApplicationError> {
        debug!("Refreshing access token");

        // Verify refresh token
        let claims = self
            .jwt_manager
            .verify_refresh_token(&request.refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token");
                AuthApplicationError::InvalidToken
            })?;

        // Get user_id from claims
        let user_id = claims.user_id().map_err(|e| {
            error!(error = %e, "Invalid user_id in token claims");
            AuthApplicationError::InvalidToken
        })?;

        // Find session by token hash
        let token_hash = self
            .token_hasher
            .hash(&request.refresh_token)
            .map_err(|_| AuthApplicationError::HashingFailed)?;
        let mut session = self
            .session_repo
            .find_by_refresh_token_hash(&token_hash.as_str())
            .await?
            .ok_or_else(|| {
                warn!("Refresh failed: session not found");
                AuthApplicationError::InvalidToken
            })?;

        if !session.is_valid() {
            warn!(session_id = %session.id(), "Refresh failed: session invalid");
            return Err(AuthApplicationError::InvalidToken);
        }

        // Update last_used_at
        session.mark_as_used();
        self.session_repo.update(&session).await?;

        // Generate new access token
        let access_token = self
            .jwt_manager
            .create_access_token(user_id, &claims.email, ACCESS_TOKEN_EXPIRY_HOURS)
            .map_err(|e| AuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        debug!(user_id = %user_id, "Token refreshed");

        Ok(AuthResponse::new(
            access_token,
            request.refresh_token, // Return same refresh token
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

        let revoked_count = if all_devices {
            // Revoke all sessions
            self.session_repo.revoke_all_by_user_id(user_id).await?
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
