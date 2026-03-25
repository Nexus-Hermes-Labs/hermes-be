pub mod channel_state;
pub mod shared_state;

pub use channel_state::ChannelState;
pub use shared_state::SharedState;

use crate::application::ports::guild_client::GuildClient;
use axum::extract::FromRef;
use std::sync::Arc;

/// Combined application state shared across all Axum handlers.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Channel domain services
    pub channel: ChannelState,
    /// Shared infrastructure (DB, Redis, gRPC clients)
    pub shared: SharedState,
}

impl FromRef<AppState> for Arc<dyn GuildClient> {
    fn from_ref(state: &AppState) -> Self {
        state.shared.guild_grpc_client.clone()
    }
}
