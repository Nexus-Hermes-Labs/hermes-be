mod user;
mod health;

use crate::presentation::http::state::AppState;
use axum::{middleware, Router};
use common::observability::HealthCheck;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use common::middleware::authentication::auth_middleware;

pub fn create_router(
    app_state: AppState,
    health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    >,
) -> Router {
    // Protected routes
   let user_routes = user::protected_routes();

    // API routes
    let api_routes = Router::new()
        .nest("/users", user_routes)
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Complete router
    Router::new()
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/auth", api_routes)
        .layer(cors)
        .layer(trace_layer)
}