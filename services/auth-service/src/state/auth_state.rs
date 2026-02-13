use crate::application::services::authentication::service::AuthService;
use crate::application::services::authentication::user_profile_client::UserProfileClient;
use crate::domain::auth_credential::{AuthCredentialRepository, PasswordService};
use crate::domain::auth_session::TokenHasher;
use crate::domain::AuthSessionRepository;
use common::infrastructure::messaging::EventPublisher;
use common::infrastructure::security::jwt_manager::JwtManager;
use std::sync::Arc;

/// Authentication domain state
///
/// Contains pre-composed authentication service and its dependencies.
/// Generic over trait implementations to maintain dependency inversion.
#[derive(Clone)]
pub struct AuthState<CR, SR, PS, TH, EP, UPC>
where
    CR: AuthCredentialRepository,
    SR: AuthSessionRepository,
    PS: PasswordService,
    TH: TokenHasher,
    EP: EventPublisher,
    UPC: UserProfileClient,
{
    /// High-level authentication service (orchestrates auth workflows)
    pub service: Arc<AuthService<CR, SR, PS, TH, EP, UPC>>,

    /// Direct access to JWT manager (for token operations in middleware/handlers)
    pub jwt_manager: Arc<JwtManager>,

    /// Direct repository access (optional, for specific handler needs)
    pub credential_repo: Arc<CR>,
    pub session_repo: Arc<SR>,
}

impl<CR, SR, PS, TH, EP, UPC> AuthState<CR, SR, PS, TH, EP, UPC>
where
    CR: AuthCredentialRepository,
    SR: AuthSessionRepository,
    PS: PasswordService,
    TH: TokenHasher,
    EP: EventPublisher,
    UPC: UserProfileClient,
{
    pub fn new(
        service: Arc<AuthService<CR, SR, PS, TH, EP, UPC>>,
        jwt_manager: Arc<JwtManager>,
        credential_repo: Arc<CR>,
        session_repo: Arc<SR>,
    ) -> Self {
        Self {
            service,
            jwt_manager,
            credential_repo,
            session_repo,
        }
    }
}
