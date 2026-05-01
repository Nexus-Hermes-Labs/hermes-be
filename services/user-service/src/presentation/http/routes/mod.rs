mod health;
mod user_me;
mod user_privacy;
mod user_profile;
mod user_relationship;

use crate::presentation::http::docs::ApiDoc;
use crate::state::AppState;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
};
use common::middleware::authorization::require_admin;
use common::observability::{
    metrics_routes, HealthCheck, HermesTraceLayer, PropagateRequestIdResponseLayer,
    RequestIdScopeLayer,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Matches \uD800-\uDFFF surrogate escape sequences in raw JSON bytes
static SURROGATE_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"\\u[dD][89a-fA-F][0-9a-fA-F]{2}").unwrap());

async fn reject_percent_encoded_paths(request: Request<Body>, next: Next) -> Response {
    if request.uri().path().contains('%') {
        return (
            StatusCode::BAD_REQUEST,
            "Path parameters must not be percent-encoded",
        )
            .into_response();
    }
    next.run(request).await
}

async fn reject_json_surrogates(request: Request<Body>, next: Next) -> Response {
    let is_json = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.starts_with("application/json"))
        .unwrap_or(false);

    if !is_json {
        return next.run(request).await;
    }

    let (parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response(),
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        if SURROGATE_RE.is_match(body_str) {
            return (
                StatusCode::BAD_REQUEST,
                "Request body must not contain surrogate escape sequences",
            )
                .into_response();
        }
    }

    let request = Request::from_parts(parts, Body::from(bytes));
    next.run(request).await
}

pub fn create_router(
    app_state: AppState,
    health_check: Arc<HealthCheck>,
    cors: CorsLayer,
    trace_layer: HermesTraceLayer,
) -> Router {
    // @me routes — RequestUser extractor enforces 401 when headers are missing
    let me_routes = user_me::user_me_routes();

    // Read-only /:user_id routes — Option<RequestUser> makes auth optional
    let profile_read_routes = user_profile::user_profile_read_routes();

    // Admin-only routes — require_admin middleware checks X-User-Role header
    let admin_routes = Router::new()
        .merge(user_profile::user_profile_admin_routes())
        .merge(user_privacy::user_privacy_routes())
        .merge(user_relationship::user_relationship_routes())
        .route_layer(axum::middleware::from_fn(require_admin));

    let user_routes = Router::new()
        .merge(me_routes)
        .merge(profile_read_routes)
        .merge(admin_routes);

    let api_router = Router::new()
        .nest("/users", user_routes)
        .with_state(app_state);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health::routes().with_state(health_check))
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(axum::middleware::from_fn(reject_json_surrogates))
        .layer(axum::middleware::from_fn(reject_percent_encoded_paths))
        .layer(trace_layer)
        .layer(PropagateRequestIdResponseLayer)
        .layer(RequestIdScopeLayer)
        .nest("/metrics", metrics_routes())
}
