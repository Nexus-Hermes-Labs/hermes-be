pub mod shared_state;
pub mod user_state;

pub use shared_state::SharedState;
pub use user_state::UserState;

/// Application state shared across HTTP handlers and gRPC services
#[derive(Clone)]
pub struct AppState {
    pub user: UserState,
    pub shared: SharedState,
}
