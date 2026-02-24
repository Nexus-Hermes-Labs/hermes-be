mod auth;
mod health;
mod internal;

use crate::presentation::http::docs::ApiDoc;
use crate::state::app_state::AppState;
use axum::Router;
use common::observability::HealthCheck;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(
    app_state: AppState,
    health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    >,
) -> Router {
    // Public routes
    let public_routes = Router::new().nest("/auth", auth::public_routes());

    // Combine and add state
    let api_routes = Router::new()
        .merge(public_routes)
        .with_state(app_state.clone());

    // Complete router
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
}
