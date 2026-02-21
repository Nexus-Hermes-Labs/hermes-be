mod health;
mod user_me;
mod user_privacy;
mod user_profile;
mod user_relationship;

use crate::presentation::http::docs::ApiDoc;
use crate::state::AppState;
use axum::Router;
use common::middleware::authentication::auth_middleware;
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
    // @me routes require JWT authentication
    let me_routes = user_me::user_me_routes().route_layer(axum::middleware::from_fn_with_state(
        app_state.shared.jwt_manager.clone(),
        auth_middleware,
    ));

    // Existing :user_id routes (admin/cross-service use)
    let id_routes = Router::new()
        .merge(user_profile::user_profile_routes())
        .merge(user_privacy::user_privacy_routes())
        .merge(user_relationship::user_relationship_routes());

    let user_routes = Router::new().merge(me_routes).merge(id_routes);

    let api_router = Router::new()
        .nest("/users", user_routes)
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(trace_layer)
}
