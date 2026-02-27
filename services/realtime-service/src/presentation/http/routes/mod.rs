//! HTTP + WebSocket route composition.

mod health;

use crate::presentation::ws::handler::ws_handler;
use crate::state::AppState;
use axum::Router;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

/// Build the top-level [`Router`] that serves both HTTP and WebSocket traffic.
pub fn create_router(app_state: AppState, cors: CorsLayer) -> Router {
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(tracing::Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        );

    Router::new()
        // WebSocket upgrade endpoint — token validated inside the handler
        .route("/ws", axum::routing::get(ws_handler))
        .with_state(app_state)
        .nest("/health", health::routes())
        .layer(cors)
        .layer(trace)
}
