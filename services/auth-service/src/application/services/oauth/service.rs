use std::sync::Arc;

use common::domain::event::IntoEventEnvelope;
use common::infrastructure::outbox::{AggregateType, NewOutboxEvent, OutboxEventType};
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::security::jwt_manager::JwtManager;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tracing::{info, warn};
use uuid::Uuid;

use super::error::OAuthApplicationError;
use crate::application::events::UserCreatedEvent;
use crate::application::ports::oauth_provider::{GoogleOAuthClient, OAuthUserInfo};
use crate::application::ports::oauth_state_store::{OAuthFlowState, OAuthStateStore};
use crate::application::ports::unit_of_work::AuthUnitOfWorkFactory;
use crate::domain::auth_credential::{AuthCredential, AuthCredentialRepository, Email, PasswordService};
use crate::domain::auth_session::{AuthSession, TokenHasher};
use crate::domain::oauth_account::{OAuthAccount, OAuthAccountRepository, OAuthProvider};
use crate::presentation::http::dto::{AuthResponse, ClientInfo};

// ============================================
// CONFIGURATION
// ============================================

const ACCESS_TOKEN_EXPIRY_HOURS: i64 = 6;
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;
/// How long an in-flight OAuth authorization may sit before the callback must
/// arrive. Bounds the Redis-stored PKCE verifier lifetime.
const STATE_TTL_SECONDS: u64 = 600;

/// What the callback must persist when issuing the login, beyond the session.
#[derive(Debug, Clone, Copy)]
enum LinkPlan {
    /// Account already linked — only a new session is written.
    None,
    /// Email matched an existing credential — insert the oauth link.
    LinkExisting,
    /// Brand-new user — create the verified credential, link, and emit the
    /// `user.created` outbox event.
    CreateNew,
}

// ============================================
// OAUTH SERVICE
// ============================================

/// Orchestrates the Google OAuth login flow (Architecture A: the frontend owns
/// the `redirect_uri` and brokers the `code` back to this service).
///
/// Token + session issuance mirrors [`AuthService::login`](crate::application::services::authentication::service::AuthService::login).
pub struct OAuthService {
    service_name: String,
    oauth_account_repo: Arc<dyn OAuthAccountRepository>,
    credential_repo: Arc<dyn AuthCredentialRepository>,
    uow_factory: Arc<dyn AuthUnitOfWorkFactory>,
    password_service: Arc<dyn PasswordService>,
    token_hasher: Arc<dyn TokenHasher>,
    jwt_manager: Arc<JwtManager>,
    state_store: Arc<dyn OAuthStateStore>,
    google_client: Arc<dyn GoogleOAuthClient>,
}

impl OAuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service_name: impl Into<String>,
        oauth_account_repo: Arc<dyn OAuthAccountRepository>,
        credential_repo: Arc<dyn AuthCredentialRepository>,
        uow_factory: Arc<dyn AuthUnitOfWorkFactory>,
        password_service: Arc<dyn PasswordService>,
        token_hasher: Arc<dyn TokenHasher>,
        jwt_manager: Arc<JwtManager>,
        state_store: Arc<dyn OAuthStateStore>,
        google_client: Arc<dyn GoogleOAuthClient>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            oauth_account_repo,
            credential_repo,
            uow_factory,
            password_service,
            token_hasher,
            jwt_manager,
            state_store,
            google_client,
        }
    }

    // ============================================
    // AUTHORIZE
    // ============================================

    /// Start a Google authorization: mint a CSRF `state` + PKCE pair, stash the
    /// verifier in the state store, and return the consent URL for the SPA to
    /// navigate to.
    pub async fn start_google_authorization(&self) -> Result<String, OAuthApplicationError> {
        let state = random_token(32);
        let redirect = self.google_client.build_authorize_url(&state)?;

        self.state_store
            .save(
                &state,
                &OAuthFlowState {
                    code_verifier: redirect.code_verifier,
                },
                STATE_TTL_SECONDS,
            )
            .await?;

        info!("Google OAuth authorization started");
        Ok(redirect.url)
    }

    // ============================================
    // CALLBACK
    // ============================================

    /// Complete a Google login from the brokered `{code, state}`. Returns the
    /// same token payload as password login.
    pub async fn complete_google_login(
        &self,
        code: String,
        state: String,
        client_info: ClientInfo,
    ) -> Result<AuthResponse, OAuthApplicationError> {
        let provider = OAuthProvider::Google;

        // 1. Validate + consume the CSRF state, recovering the PKCE verifier.
        let flow = self
            .state_store
            .take(&state)
            .await?
            .ok_or(OAuthApplicationError::InvalidState)?;

        // 2/3. Exchange the code and fetch the verified profile.
        let provider_token = self
            .google_client
            .exchange_code(&code, &flow.code_verifier)
            .await?;
        let info = self.google_client.fetch_user_info(&provider_token).await?;

        // 4. We only auto-link/create on a provider-verified email.
        if !info.email_verified {
            warn!("Google login rejected: email not verified by provider");
            return Err(OAuthApplicationError::EmailNotVerifiedByProvider);
        }

        let email = Email::new(&info.email)
            .map_err(|_| OAuthApplicationError::InvalidEmail(info.email.clone()))?;

        // 5. Resolve the local credential and decide what to persist.
        let (credential, plan) = self.resolve_credential(provider, &info, &email).await?;

        self.issue_login(provider, credential, plan, &info, client_info)
            .await
    }

    /// Map a Google profile onto a local credential, deciding the write plan.
    async fn resolve_credential(
        &self,
        provider: OAuthProvider,
        info: &OAuthUserInfo,
        email: &Email,
    ) -> Result<(AuthCredential, LinkPlan), OAuthApplicationError> {
        // (a) Already linked → load and reuse.
        if let Some(link) = self
            .oauth_account_repo
            .find_by_provider_and_subject(provider, &info.provider_user_id)
            .await?
        {
            let credential = self
                .credential_repo
                .find_by_id(link.credential_id())
                .await?
                .ok_or_else(|| {
                    OAuthApplicationError::Internal(format!(
                        "oauth link {} points to a missing credential",
                        link.id()
                    ))
                })?;
            Self::ensure_usable(&credential)?;
            return Ok((credential, LinkPlan::None));
        }

        // (b) Verified email matches an existing account → auto-link.
        if let Some(credential) = self.credential_repo.find_by_email(email).await? {
            Self::ensure_usable(&credential)?;
            info!(user_id = %credential.user_id(), "Linking Google account to existing credential");
            return Ok((credential, LinkPlan::LinkExisting));
        }

        // (c) Brand-new user → verified OAuth-only credential with a random hash.
        let user_id = Uuid::new_v4();
        let random_pw = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
        let password_hash = self.password_service.hash_password(&random_pw).map_err(|_| {
            OAuthApplicationError::Internal("failed to hash OAuth placeholder password".to_string())
        })?;
        let credential = AuthCredential::new_oauth(user_id, email.clone(), password_hash);
        info!(user_id = %credential.user_id(), "Creating new credential from Google login");
        Ok((credential, LinkPlan::CreateNew))
    }

    /// Mint tokens + a session and persist the plan's writes atomically.
    async fn issue_login(
        &self,
        provider: OAuthProvider,
        credential: AuthCredential,
        plan: LinkPlan,
        info: &OAuthUserInfo,
        client_info: ClientInfo,
    ) -> Result<AuthResponse, OAuthApplicationError> {
        let access_token = self
            .jwt_manager
            .create_access_token(
                credential.user_id(),
                credential.email().as_str(),
                credential.system_role(),
                ACCESS_TOKEN_EXPIRY_HOURS,
            )
            .map_err(|e| OAuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(
                credential.user_id(),
                credential.email().as_str(),
                REFRESH_TOKEN_EXPIRY_DAYS,
            )
            .map_err(|e| OAuthApplicationError::TokenGenerationFailed(e.to_string()))?;

        let token_hash = self
            .token_hasher
            .hash(&refresh_token)
            .map_err(|_| OAuthApplicationError::TokenGenerationFailed("refresh hash".to_string()))?;

        let session = AuthSession::create(
            credential.id(),
            token_hash.as_str().to_owned(),
            REFRESH_TOKEN_EXPIRY_DAYS,
            client_info.ip_address.clone(),
            client_info.user_agent.clone(),
            client_info.device_id.clone(),
        );

        let oauth_account = OAuthAccount::new(
            credential.id(),
            provider,
            info.provider_user_id.clone(),
            info.email.clone(),
        );

        let outbox_event = match plan {
            LinkPlan::CreateNew => Some(self.build_user_created_event(&credential, info)?),
            LinkPlan::None | LinkPlan::LinkExisting => None,
        };

        let credential_for_tx = credential.clone();
        let session_for_tx = session.clone();
        let oauth_account_for_tx = oauth_account.clone();

        let tx_result = self
            .uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    match plan {
                        LinkPlan::None => {}
                        LinkPlan::LinkExisting => {
                            uow.oauth_accounts().save(&oauth_account_for_tx).await?;
                        }
                        LinkPlan::CreateNew => {
                            uow.credentials().save_verified(&credential_for_tx).await?;
                            uow.oauth_accounts().save(&oauth_account_for_tx).await?;
                            if let Some(event) = &outbox_event {
                                uow.outbox().save(event).await?;
                            }
                        }
                    }
                    uow.sessions().save(&session_for_tx).await?;
                    Ok(())
                })
            }))
            .await;

        match tx_result {
            Ok(()) => {
                info!(user_id = %credential.user_id(), "Google login successful");
                Ok(AuthResponse::new(
                    access_token,
                    refresh_token,
                    ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60,
                ))
            }
            Err(e) if is_oauth_unique_violation(&e) => {
                // A concurrent first-login linked this provider account between
                // our read and write. Re-fetch the winner and issue a session
                // only (no further link writes → no second conflict).
                warn!("Concurrent Google account link detected; recovering");
                self.recover_concurrent_link(provider, info, client_info)
                    .await
            }
            Err(e) => Err(OAuthApplicationError::RepositoryError(e.to_string())),
        }
    }

    /// Re-resolve after a unique-violation race and issue a session for the
    /// credential the racing request linked.
    async fn recover_concurrent_link(
        &self,
        provider: OAuthProvider,
        info: &OAuthUserInfo,
        client_info: ClientInfo,
    ) -> Result<AuthResponse, OAuthApplicationError> {
        let link = self
            .oauth_account_repo
            .find_by_provider_and_subject(provider, &info.provider_user_id)
            .await?
            .ok_or_else(|| {
                OAuthApplicationError::Internal(
                    "oauth link vanished during concurrent-login recovery".to_string(),
                )
            })?;
        let credential = self
            .credential_repo
            .find_by_id(link.credential_id())
            .await?
            .ok_or_else(|| {
                OAuthApplicationError::Internal(
                    "credential missing during concurrent-login recovery".to_string(),
                )
            })?;
        Self::ensure_usable(&credential)?;
        // Box the recursive call: the `None` plan writes no oauth link, so this
        // cannot re-enter the conflict path, but the compiler needs the
        // indirection to size the future.
        Box::pin(self.issue_login(provider, credential, LinkPlan::None, info, client_info)).await
    }

    /// Reject suspended/deleted accounts. Failed-password lockout is a
    /// password concern and does not block provider-verified OAuth login.
    fn ensure_usable(credential: &AuthCredential) -> Result<(), OAuthApplicationError> {
        if credential.is_deleted() {
            return Err(OAuthApplicationError::AccountDeleted);
        }
        if credential.account_status().is_suspended() {
            return Err(OAuthApplicationError::AccountSuspended);
        }
        Ok(())
    }

    /// Build the `user.created` outbox event for a freshly created OAuth user so
    /// user-service provisions a profile (same contract as registration).
    fn build_user_created_event(
        &self,
        credential: &AuthCredential,
        info: &OAuthUserInfo,
    ) -> Result<NewOutboxEvent, OAuthApplicationError> {
        let username = derive_username(credential.email());
        let display_name = info
            .display_name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| username.clone());

        let event = UserCreatedEvent::new(
            credential.user_id(),
            credential.email().as_str().to_string(),
            username,
            display_name,
        );
        let envelope = event.into_envelope(&self.service_name);

        Ok(NewOutboxEvent {
            id: envelope.event_id,
            aggregate_id: envelope.aggregate_id,
            aggregate_type: AggregateType::User,
            event_type: OutboxEventType::UserCreated,
            payload: serde_json::to_value(&envelope)
                .map_err(|e| OAuthApplicationError::Internal(e.to_string()))?,
        })
    }
}

// ============================================
// HELPERS
// ============================================

/// Random URL-safe alphanumeric token of `len` characters.
fn random_token(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// Synthesize a username for a new OAuth user from the email local-part plus a
/// short random suffix. user-service owns final uniqueness; the suffix just
/// keeps collisions rare. A future "choose username" step can refine this.
fn derive_username(email: &Email) -> String {
    let local = email.as_str().split('@').next().unwrap_or("user");
    let cleaned: String = local
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
        .collect();
    let base = if cleaned.is_empty() {
        "user".to_string()
    } else {
        cleaned
    };
    format!("{}-{}", base, random_token(4).to_lowercase())
}

/// True when a repository error is the `oauth_accounts` unique-constraint
/// violation (concurrent first-login linking the same provider account).
fn is_oauth_unique_violation(error: &RepositoryError) -> bool {
    match error {
        RepositoryError::Database(sqlx::Error::Database(db_error)) => {
            db_error.code().as_deref() == Some("23505")
                && db_error.constraint() == Some("uq_oauth_accounts_provider_subject")
        }
        RepositoryError::DuplicateEntry(_) => true,
        _ => false,
    }
}
