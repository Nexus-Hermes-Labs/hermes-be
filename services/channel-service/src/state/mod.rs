pub mod channel_state;
pub mod shared_state;

pub use channel_state::ChannelState;
pub use shared_state::SharedState;

use crate::infrastructure::grpc::GuildGrpcClient;
use axum::extract::FromRef;
use common::infrastructure::security::jwt_manager::JwtManager;
use std::sync::Arc;

/// Combined application state shared across all Axum handlers.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Channel domain services
    pub channel: ChannelState,
    /// Shared infrastructure (DB, Redis, JWT, gRPC clients)
    pub shared: SharedState,
}

impl FromRef<AppState> for Arc<JwtManager> {
    fn from_ref(state: &AppState) -> Self {
        state.shared.jwt_manager.clone()
    }
}

impl FromRef<AppState> for Arc<GuildGrpcClient> {
    fn from_ref(state: &AppState) -> Self {
        state.shared.guild_grpc_client.clone()
    }
}
