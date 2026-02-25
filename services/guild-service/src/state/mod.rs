pub mod guild_state;
pub mod shared_state;

pub use guild_state::GuildState;
pub use shared_state::SharedState;

/// Application state shared across HTTP handlers and gRPC services.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Guild-domain services (guild, member, role, invite).
    pub guild: GuildState,
    /// Cross-cutting infrastructure (database, Redis, metrics).
    pub shared: SharedState,
}
