use crate::state::channel_state::ChannelState;
use crate::state::shared_state::SharedState;

#[derive(Clone)]
pub struct AppState {
    pub channel: ChannelState,
    pub shared: SharedState,
}
