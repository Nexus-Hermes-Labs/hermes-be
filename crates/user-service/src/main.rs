use axum::{
    routing::{get, patch, post, delete},
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
        .route("/users/@me", get(get_current_user).patch(update_profile))
        .route("/users/:id", get(get_user))
        .route("/users/search", get(search_users))
        .route("/users/@me/friends", get(get_friends).post(add_friend))
        .route("/users/@me/friends/:id", delete(remove_friend))
        .route("/users/@me/blocked", post(block_user))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    tracing::info!("User Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn get_current_user() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn update_profile() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn get_user() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn search_users() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn get_friends() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn add_friend() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn remove_friend() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}

async fn block_user() -> StatusCode {
    // TODO: Implement
    StatusCode::NOT_IMPLEMENTED
}
