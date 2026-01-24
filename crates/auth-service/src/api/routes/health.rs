use crate::api::handlers;
use axum::{routing::get, Router};
use std::sync::Arc;
use common::observability::HealthCheck;

pub fn routes() -> Router<Arc<HealthCheck>> {
    Router::new()
        .route("/", get(handlers::health::health_handler))
        .route("/live", get(handlers::health::liveness_handler))
        .route("/ready", get(handlers::health::readiness_handler))
}
