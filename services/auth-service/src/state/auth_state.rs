use crate::application::services::authentication::service::AuthService;
use crate::domain::auth_credential::AuthCredentialRepository;
use crate::domain::AuthSessionRepository;
use common::infrastructure::security::jwt_manager::JwtManager;
use std::sync::Arc;

/// Authentication domain state
///
/// Contains pre-composed authentication service and its dependencies.
#[derive(Clone)]
pub struct AuthState {
    /// High-level authentication service (orchestrates auth workflows)
    pub service: Arc<AuthService>,

    /// Direct access to JWT manager (for token operations in middleware/handlers)
    pub jwt_manager: Arc<JwtManager>,

    /// Direct repository access (optional, for specific handler needs)
    pub credential_repo: Arc<dyn AuthCredentialRepository>,
    pub session_repo: Arc<dyn AuthSessionRepository>,
}

impl AuthState {
    pub fn new(
        service: Arc<AuthService>,
        jwt_manager: Arc<JwtManager>,
        credential_repo: Arc<dyn AuthCredentialRepository>,
        session_repo: Arc<dyn AuthSessionRepository>,
    ) -> Self {
        Self {
            service,
            jwt_manager,
            credential_repo,
            session_repo,
        }
    }
}
