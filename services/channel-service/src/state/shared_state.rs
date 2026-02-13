use sqlx::PgPool;
use common::observability::Metrics;

#[derive(Clone)]
pub struct SharedState {
    pub db: PgPool,
    pub metrics: Metrics,
}
