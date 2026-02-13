mod user_privacy;
mod user_profile;

use crate::state::AppState;
use axum::Router;
use common::observability::HealthCheck;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn create_router(
    app_state: AppState,
    _health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    >,
) -> Router {
    let user_routes = Router::new()
        .merge(user_profile::user_profile_routes())
        .merge(user_privacy::user_privacy_routes())
        .with_state(app_state);

    Router::new()
        .nest("/api/users", user_routes)
        .layer(cors)
        .layer(trace_layer)
}
