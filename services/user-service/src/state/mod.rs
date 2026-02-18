pub mod shared_state;
pub mod user_state;

pub use shared_state::SharedState;
pub use user_state::UserState;

use axum::extract::FromRef;
use common::infrastructure::security::jwt_manager::JwtManager;
use std::sync::Arc;

/// Application state shared across HTTP handlers and gRPC services
#[derive(Clone)]
pub struct AppState {
    pub user: UserState,
    pub shared: SharedState,
}

impl FromRef<AppState> for Arc<JwtManager> {
    fn from_ref(state: &AppState) -> Self {
        state.shared.jwt_manager.clone()
    }
}
