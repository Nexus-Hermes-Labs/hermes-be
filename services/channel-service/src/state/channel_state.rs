use crate::application::ChannelService;
use std::sync::Arc;

/// Channel domain state
///
/// Holds pre-composed channel application services, shared across handlers via `AppState`.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct ChannelState {
    /// Handles channel CRUD operations.
    pub channel_service: Arc<ChannelService>,
}

impl ChannelState {
    /// Compose a `ChannelState` from a pre-built application service.
    #[must_use]
    pub const fn new(channel_service: Arc<ChannelService>) -> Self {
        Self { channel_service }
    }
}
