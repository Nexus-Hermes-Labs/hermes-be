use crate::infrastructure::persistence::postgres::{
    PostgresAuthCredentialRepository, PostgresAuthSessionRepository,
};
use crate::infrastructure::security::password::argon2_service::Argon2PasswordService;
use crate::infrastructure::security::token::sha256_service::Sha256TokenHasher;
use crate::state::auth_state::AuthState;
use crate::state::shared_state::SharedState;
use common::infrastructure::messaging::NatsEventPublisher;

/// Application-wide state container
///
/// Composed of domain-specific states. This type is concrete because
/// it's only used in the composition root (main.rs).
#[derive(Clone)]
pub struct AppState {
    pub auth: AuthState<
        PostgresAuthCredentialRepository,
        PostgresAuthSessionRepository,
        Argon2PasswordService,
        Sha256TokenHasher,
        NatsEventPublisher,
    >,
    pub shared: SharedState,
}
