use axum::{
    routing::{get, post, patch, delete},
    Router,
    http::StatusCode,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use common::AppState;

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
    let state = Arc::new(app_state);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/channels/:id/messages", get(get_messages).post(send_message))
        .route("/messages/:id", patch(edit_message).delete(delete_message))
        .route("/messages/:id/reactions/:emoji", post(add_reaction).delete(remove_reaction))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8084".to_string()).parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Chat Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> StatusCode { StatusCode::OK }
async fn get_messages() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn send_message() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn edit_message() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_message() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn add_reaction() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn remove_reaction() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
