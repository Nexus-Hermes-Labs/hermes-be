pub mod messaging_state;
pub mod shared_state;

pub use messaging_state::MessagingState;
pub use shared_state::SharedState;

/// Application state shared across HTTP handlers and gRPC services.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Messaging domain services (messages, reactions, conversations).
    pub messaging: MessagingState,
    /// Cross-cutting infrastructure (database, Redis, metrics, NATS).
    pub shared: SharedState,
}
