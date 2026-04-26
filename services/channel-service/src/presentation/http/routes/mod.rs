mod channel;

use crate::presentation::http::docs::ApiDoc;
use crate::state::AppState;
use axum::Router;
use common::observability::{HealthCheck, HermesTraceLayer};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(
    app_state: AppState,
    _health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: HermesTraceLayer,
) -> Router {
    // Mutating routes — identity injected by Traefik ForwardAuth via RequestUser extractor
    let authenticated_routes = Router::new().merge(channel::authenticated_channel_routes());

    // Public routes (read-only channel queries)
    let public_routes = channel::public_channel_routes();

    let api_router = Router::new()
        .merge(authenticated_routes)
        .merge(public_routes)
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(trace_layer)
}
