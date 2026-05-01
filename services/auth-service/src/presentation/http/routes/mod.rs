mod auth;
mod health;
mod internal;

use crate::presentation::http::docs::ApiDoc;
use crate::state::app_state::AppState;
use axum::Router;
use common::observability::{
    metrics_routes, HealthCheck, HermesTraceLayer, PropagateRequestIdResponseLayer,
    RequestIdScopeLayer,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(
    app_state: AppState,
    health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: HermesTraceLayer,
) -> Router {
    // Public routes
    let public_routes = Router::new().nest("/auth", auth::public_routes());

    // Combine and add state
    let api_routes = Router::new()
        .merge(public_routes)
        .with_state(app_state.clone());

    // Complete router. `/metrics` is mounted *after* the layer chain so
    // Prometheus scrapes don't generate request-id spans or get counted in
    // `http_requests_total` themselves.
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health::routes().with_state(health_check))
        .nest(
            "/internal",
            internal::internal_routes().with_state(app_state),
        )
        .nest("/api/v1", api_routes)
        .layer(cors)
        .layer(trace_layer)
        .layer(PropagateRequestIdResponseLayer)
        .layer(RequestIdScopeLayer)
        .nest("/metrics", metrics_routes())
}
