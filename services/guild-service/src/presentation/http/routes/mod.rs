mod guild;
mod guild_invite;
mod guild_member;
mod guild_role;

use crate::state::AppState;
use axum::Router;
use common::middleware::authentication::auth_middleware;
use common::observability::HealthCheck;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::presentation::http::docs::ApiDoc;

pub fn create_router(
    app_state: AppState,
    _health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    >,
) -> Router {
    // All guild routes require JWT authentication
    let authenticated_guild_routes = Router::new()
        .merge(guild::guild_routes())
        .merge(guild_member::guild_member_routes())
        .merge(guild_role::guild_role_routes())
        .merge(guild_invite::guild_invite_routes())
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.shared.jwt_manager.clone(),
            auth_middleware,
        ));

    // Invite lookup is public (no auth required to view an invite)
    let public_routes = guild_invite::public_invite_routes();

    let api_router = Router::new()
        .merge(authenticated_guild_routes)
        .merge(public_routes)
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(trace_layer)
}
