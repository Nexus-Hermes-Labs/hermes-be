use axum::{routing::get, Router};
use common::observability::HealthCheck;
use std::sync::Arc;

pub fn routes() -> Router<Arc<HealthCheck>> {
    Router::new()
        .route("/", get(common::observability::health::health_handler))
        .route("/live", get(common::observability::health::liveness_handler))
        .route("/ready", get(common::observability::health::readiness_handler))
}
