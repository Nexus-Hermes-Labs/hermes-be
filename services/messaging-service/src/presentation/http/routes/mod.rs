mod conversation;
mod health;
mod message;
mod reaction;

use crate::presentation::http::docs::ApiDoc;
use crate::state::AppState;
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
    let api_router = Router::new()
        .merge(message::channel_message_routes())
        .merge(message::conversation_message_routes())
        .merge(message::message_routes())
        .merge(reaction::reaction_routes())
        .merge(conversation::conversation_routes())
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(trace_layer)
}
