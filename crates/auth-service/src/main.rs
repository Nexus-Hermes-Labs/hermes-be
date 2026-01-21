use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use common::{AppState, Result, AppError};

#[derive(Clone)]
struct AuthState {
    app_state: AppState,
    jwt_secret: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    user: UserInfo,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: Uuid,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://hermes:hermes_dev_password@localhost:5432/hermes".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:redis_dev_password@localhost:6379".to_string());
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret".to_string());

    // Initialize app state
    let app_state = AppState::new(&database_url, &redis_url, &nats_url).await?;
    
    let state = Arc::new(AuthState {
        app_state,
        jwt_secret,
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    tracing::info!("Auth Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn register(
    State(_state): State<Arc<AuthState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::info!("Registering user: {}", payload.username);
    
    // TODO: Implement registration logic
    // 1. Validate input
    // 2. Hash password with Argon2
    // 3. Insert user into database
    // 4. Generate JWT tokens
    // 5. Publish UserCreated event
    
    Err(AppError::InternalServerError("Not implemented".to_string()))
}

async fn login(
    State(_state): State<Arc<AuthState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::info!("Login attempt for: {}", payload.email);
    
    // TODO: Implement login logic
    // 1. Find user by email
    // 2. Verify password with Argon2
    // 3. Generate JWT tokens
    // 4. Store refresh token in Redis
    
    Err(AppError::InternalServerError("Not implemented".to_string()))
}

async fn refresh_token(
    State(_state): State<Arc<AuthState>>,
) -> Result<Json<AuthResponse>> {
    // TODO: Implement refresh token logic
    Err(AppError::InternalServerError("Not implemented".to_string()))
}

async fn logout(
    State(_state): State<Arc<AuthState>>,
) -> Result<StatusCode> {
    // TODO: Implement logout logic
    // Invalidate refresh token in Redis
    Ok(StatusCode::OK)
}
