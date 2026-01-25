use anyhow::{Context, Result};
use tracing::info;

use crate::api::server::Server;
use common::config::{Config, CONFIG};
use common::observability;

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    // ============================================
    // 1. Load Configuration
    // ============================================
    Config::load().context("Failed to load configuration")?;

    // ============================================
    // 2. Initialize Observability
    // ============================================
    observability::tracing::init_tracing(
        &CONFIG.logging,
        &CONFIG.app.name,
        &CONFIG.app.environment.to_string(),
    )
    .context("Failed to initialize tracing")?;

    info!("🚀 Starting {} v{}", CONFIG.app.name, CONFIG.app.version);
    info!("📍 Environment: {}", CONFIG.app.environment);
    info!("🌐 Host: {}:{}", CONFIG.app.host, CONFIG.app.port);

    let metrics =
        observability::metrics::Metrics::init().context("Failed to initialize metrics")?;

    // ============================================
    // 3. Initialize Database
    // ============================================
    info!("📦 Connecting to PostgreSQL...");
    let db_pool = infrastructure::db::create_pool(&CONFIG.database)
        .await
        .context("Failed to connect to database")?;
    info!("✅ Database connected");

    // ============================================
    // 4. Initialize Redis
    // ============================================
    info!("🔴 Connecting to Redis...");
    let redis_client =
        redis::Client::open(CONFIG.redis.url.clone()).context("Failed to create Redis client")?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .context("Failed to connect to Redis")?;
    info!("✅ Redis connected");

    // ============================================
    // 5. Start Server
    // ============================================
    info!("🎯 Auth Service ready!");

    let server = Server::new(&CONFIG, db_pool, redis_manager, metrics)
        .await
        .context("Failed to initialize server")?;

    // Server içinde zaten graceful shutdown var
    server.run().await.context("Server error")?;

    // ============================================
    // 6. Cleanup
    // ============================================
    info!("👋 Auth Service stopped");
    observability::tracing::shutdown_tracing();

    Ok(())
}
