pub mod chat_state;
pub mod shared_state;

pub use chat_state::ChatState;
pub use shared_state::SharedState;

/// Application state shared across HTTP handlers and gRPC services.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Chat-domain services (messages, reactions).
    pub chat: ChatState,
    /// Cross-cutting infrastructure (database, Redis, metrics).
    pub shared: SharedState,
}
