//! HTTP + WebSocket route composition.

mod health;

use crate::presentation::ws::handler::ws_handler;
use crate::state::AppState;
use axum::Router;
use common::observability::{
    metrics_routes, request_trace_layer, PropagateRequestIdResponseLayer, RequestIdScopeLayer,
};
use tower_http::cors::CorsLayer;

/// Build the top-level [`Router`] that serves both HTTP and WebSocket traffic.
pub fn create_router(app_state: AppState, cors: CorsLayer) -> Router {
    let trace = request_trace_layer();

    Router::new()
        // WebSocket upgrade endpoint — token validated inside the handler
        .route("/ws", axum::routing::get(ws_handler))
        .with_state(app_state)
        .nest("/health", health::routes())
        .layer(cors)
        .layer(trace)
        .layer(PropagateRequestIdResponseLayer)
        .layer(RequestIdScopeLayer)
        .nest("/metrics", metrics_routes())
}
