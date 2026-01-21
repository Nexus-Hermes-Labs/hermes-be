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
        .route("/servers", get(list_servers).post(create_server))
        .route("/servers/:id", get(get_server).patch(update_server).delete(delete_server))
        .route("/servers/:id/channels", post(create_channel))
        .route("/channels/:id", get(get_channel).patch(update_channel).delete(delete_channel))
        .route("/servers/:id/members", get(get_members))
        .route("/servers/:id/roles", post(create_role))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8083".to_string()).parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Channel Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> StatusCode { StatusCode::OK }
async fn list_servers() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn create_server() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn get_server() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn update_server() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_server() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn create_channel() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn get_channel() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn update_channel() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_channel() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn get_members() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn create_role() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
