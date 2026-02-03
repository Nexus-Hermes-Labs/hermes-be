mod user;
mod health;

use crate::api::state::AppState;
use axum::Router;
use common::observability::HealthCheck;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn create_router(
    app_state: AppState,
    health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    >,
) -> Router {
    // Protected routes
    let protected_routes = Router::new().nest("/user", user::protected_routes());

    // Combine and add state
    let api_routes = Router::new().merge(protected_routes).with_state(app_state);

    // Complete router
    Router::new()
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/v1", api_routes)
        .layer(cors)
        .layer(trace_layer)
}
