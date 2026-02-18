use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::Metrics;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub metrics: Metrics,
    pub jwt_manager: Arc<JwtManager>,
}
