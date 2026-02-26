use common::observability::Metrics;
use sqlx::PgPool;

/// Shared infrastructure state injected into every HTTP handler and gRPC service.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SharedState {
    /// `PostgreSQL` connection pool for all database operations.
    pub db: PgPool,
    /// Redis async connection manager.
    pub redis: redis::aio::ConnectionManager,
    /// Prometheus metrics collector.
    pub metrics: Metrics,
    /// NATS async client (kept for potential direct publishing from handlers).
    pub nats: async_nats::Client,
}
