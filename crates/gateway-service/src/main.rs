use axum::{
    routing::{get, any},
    Router,
    http::StatusCode,
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use common::AppState;

#[derive(Clone)]
struct GatewayState {
    app_state: AppState,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://hermes:hermes_dev_password@localhost:5432/hermes".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:redis_dev_password@localhost:6379".to_string());
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let app_state = AppState::new(&database_url, &redis_url, &nats_url).await?;
    
    let state = Arc::new(GatewayState { app_state });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(websocket_handler))
        .route("/api/*path", any(proxy_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    tracing::info!("Gateway Service listening on {}", addr);
    tracing::info!("WebSocket available at: ws://localhost:{}/ws", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("WebSocket connection established");
    
    // TODO: Implement WebSocket message handling
    // 1. Authenticate connection
    // 2. Send READY event
    // 3. Handle incoming messages (heartbeat, voice state, etc.)
    // 4. Subscribe to NATS events and forward to client
    
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                tracing::debug!("Received WebSocket message: {:?}", msg);
                // Handle message
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    tracing::info!("WebSocket connection closed");
}

async fn proxy_handler() -> StatusCode {
    // TODO: Implement API proxying to backend services
    // Route requests to appropriate microservices based on path
    StatusCode::NOT_IMPLEMENTED
}
