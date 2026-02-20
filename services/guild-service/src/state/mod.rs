pub mod guild_state;
pub mod shared_state;

pub use guild_state::GuildState;
pub use shared_state::SharedState;

use axum::extract::FromRef;
use common::infrastructure::security::jwt_manager::JwtManager;
use std::sync::Arc;

/// Application state shared across HTTP handlers and gRPC services.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Guild-domain services (guild, member, role, invite).
    pub guild: GuildState,
    /// Cross-cutting infrastructure (database, Redis, metrics, JWT).
    pub shared: SharedState,
}

impl FromRef<AppState> for Arc<JwtManager> {
    fn from_ref(state: &AppState) -> Self {
        state.shared.jwt_manager.clone()
    }
}
