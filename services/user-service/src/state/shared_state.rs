use common::observability::Metrics;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SharedState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub metrics: Metrics,
}
