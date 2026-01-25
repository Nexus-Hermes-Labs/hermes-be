use anyhow::{Context, Result};
use tracing::info;

use crate::api::server::Server;
use common::config::{config, Config, CONFIG};
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
    let service_name = env!("CARGO_PKG_NAME");
    common::config::init_config(service_name);

    // ============================================
    // 2. Initialize Observability
    // ============================================
    observability::tracing::init_tracing(
        &config().logging,
        &config().service.name,
        &config().service.environment.to_string(),
    )
    .context("Failed to initialize tracing")?;

    info!(
        "🚀 Starting {} v{}",
        config().service.name,
        config().service.version
    );
    info!("📍 Environment: {}", config().service.environment);
    info!(
        "🌐 Host: {}:{}",
        config().service.host,
        config().service.port
    );

    let metrics =
        observability::metrics::Metrics::init().context("Failed to initialize metrics")?;

    // ============================================
    // 3. Initialize Database
    // ============================================
    info!("📦 Connecting to PostgreSQL...");
    let db_pool = infrastructure::db::create_pool(&config().database)
        .await
        .context("Failed to connect to database")?;
    info!("✅ Database connected");

    // ============================================
    // 4. Initialize Redis
    // ============================================
    info!("🔴 Connecting to Redis...");
    let redis_client =
        redis::Client::open(config().redis.url.clone()).context("Failed to create Redis client")?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .context("Failed to connect to Redis")?;
    info!("✅ Redis connected");

    // ============================================
    // 5. Start Server
    // ============================================
    info!("🎯 Auth Service ready!");

    let server = Server::new(&config(), db_pool, redis_manager, metrics)
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
