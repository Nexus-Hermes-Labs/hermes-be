use common::observability::Metrics;
use sqlx::PgPool;

/// Cross-cutting infrastructure shared across all handlers.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SharedState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub metrics: Metrics,
}
