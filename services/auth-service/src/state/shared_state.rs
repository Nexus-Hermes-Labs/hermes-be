use sqlx::PgPool;
use common::observability::Metrics;
use crate::domain::auth_credential::EmailService;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub metrics: Metrics,
    pub email_service: Arc<dyn EmailService>,
}
