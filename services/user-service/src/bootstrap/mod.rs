mod app_builder;

use anyhow::{Context, Result};
use common::config::config;
use common::observability;
use tracing::info;

pub use app_builder::AppBuilder;

/// Bootstrap and run the application
pub async fn run(service_name: &'static str) -> Result<()> {
    // ========================================
    // 1. INITIALIZE METRICS
    // ========================================
    let metrics = observability::metrics::Metrics::init()
        .context("Failed to initialize metrics")?;

    info!("✅ Metrics initialized");

    // ========================================
    // 2. INITIALIZE DATABASE
    // ========================================
    info!("📦 Connecting to PostgreSQL...");
    let db_pool = crate::infrastructure::persistence::postgres::connection::create_pool(
        &config().database
    )
    .await
    .context("Failed to connect to database")?;

    info!("✅ Database connected");

    // ========================================
    // 3. INITIALIZE REDIS
    // ========================================
    info!("🔴 Connecting to Redis...");
    let redis_client = redis::Client::open(config().redis.get_url().clone())
        .context("Failed to create Redis client")?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .context("Failed to connect to Redis")?;

    info!("✅ Redis connected");

    // ========================================
    // 4. BUILD APPLICATION
    // ========================================
    info!("🔧 Building application...");

    let (server, grpc_router) = AppBuilder::new()
        .with_service_name(service_name)
        .with_database(db_pool)
        .with_redis(redis_manager)
        .with_metrics(metrics)
        .build()
        .await
        .context("Failed to build application")?;

    info!("🎯 Application ready!");

    // ========================================
    // 5. RUN SERVERS (HTTP + gRPC concurrently)
    // ========================================
    info!(
        "🌐 Starting HTTP server on {}:{}",
        config().service.host,
        config().service.port
    );
    info!(
        "🔗 Starting gRPC server on {}:{}",
        config().service.host,
        config().service.grpc_port
    );

    let http_handle = tokio::spawn(async move {
        server.run().await.context("HTTP server error")
    });

    let grpc_addr = std::net::SocketAddr::from(([0, 0, 0, 0], config().service.grpc_port));
    let grpc_handle = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(grpc_router)
            .serve(grpc_addr)
            .await
            .context("gRPC server error")
    });

    // Wait for either server to finish (or fail)
    tokio::select! {
        result = http_handle => {
            result.context("HTTP server task panicked")??;
        }
        result = grpc_handle => {
            result.context("gRPC server task panicked")??;
        }
    }

    Ok(())
}
