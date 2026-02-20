// Changed from use crate::infrastructure::email::LettreEmailService;
use crate::state::auth_state::AuthState;
use crate::state::shared_state::SharedState;
// New import

/// Application-wide state container
///
/// Composed of domain-specific states. This type is concrete because
/// it's only used in the composition root (main.rs).
#[derive(Clone)]
pub struct AppState {
    pub auth: AuthState,
    pub shared: SharedState,
}
