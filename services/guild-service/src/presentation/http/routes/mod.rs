mod guild;
mod guild_invite;
mod guild_member;
mod guild_role;
mod health;

use crate::presentation::http::docs::ApiDoc;
use crate::state::AppState;
use axum::Router;
use common::observability::{
    HealthCheck, HermesTraceLayer, PropagateRequestIdResponseLayer, RequestIdScopeLayer,
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
    // All guild routes — identity injected by Traefik ForwardAuth via RequestUser extractor
    let authenticated_guild_routes = Router::new()
        .merge(guild::guild_routes())
        .merge(guild_member::guild_member_routes())
        .merge(guild_role::guild_role_routes())
        .merge(guild_invite::guild_invite_routes());

    // Invite lookup is public (no auth required to view an invite)
    let public_routes = guild_invite::public_invite_routes();

    let api_router = Router::new()
        .merge(authenticated_guild_routes)
        .merge(public_routes)
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(trace_layer)
        .layer(PropagateRequestIdResponseLayer)
        .layer(RequestIdScopeLayer)
}
