mod app_builder;
pub mod error;

use std::sync::Arc;

use crate::bootstrap::error::BootstrapError;
use common::infrastructure::outbox::{OutboxPublisherTask, OutboxRepository, OutboxStreamConfig};
use common::observability;
use common_config::config;
use tracing::info;

pub use app_builder::AppBuilder;

/// Bootstrap and run the chat service.
pub async fn run(service_name: &'static str) -> Result<(), BootstrapError> {
    // ────────────────────────────────────────────────────────────────────
    // 1. METRICS
    // ────────────────────────────────────────────────────────────────────
    let metrics = observability::metrics::Metrics::init().map_err(|e| {
        BootstrapError::Initialization(format!("Failed to initialize metrics: {e}"))
    })?;
    info!("✅ Metrics initialized");

    // ────────────────────────────────────────────────────────────────────
    // 2. DATABASE
    // ────────────────────────────────────────────────────────────────────
    info!("📦 Connecting to PostgreSQL...");
    let db_pool =
        crate::infrastructure::persistence::postgres::connection::create_pool(&config().database)
            .await
            .map_err(BootstrapError::Database)?;
    info!("✅ Database connected");

    // ────────────────────────────────────────────────────────────────────
    // 3. REDIS
    // ────────────────────────────────────────────────────────────────────
    info!("🔴 Connecting to Redis...");
    let redis_client =
        redis::Client::open(config().redis.get_url()).map_err(BootstrapError::Redis)?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .map_err(BootstrapError::Redis)?;
    info!("✅ Redis connected");

    // ────────────────────────────────────────────────────────────────────
    // 4. NATS
    // ────────────────────────────────────────────────────────────────────
    info!("📨 Connecting to NATS...");
    let nats_url = config().nats.get_url();
    let nats_client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| BootstrapError::Infrastructure(format!("NATS connect error: {e}")))?;
    info!("✅ NATS connected");

    // ────────────────────────────────────────────────────────────────────
    // 5. BUILD APPLICATION
    // ────────────────────────────────────────────────────────────────────
    info!("🔧 Building application...");
    let (server, grpc_router) = AppBuilder::new()
        .with_service_name(service_name)
        .with_database(db_pool.clone())
        .with_redis(redis_manager)
        .with_metrics(metrics)
        .with_nats(nats_client)
        .build()?;
    info!("🎯 Application ready!");

    // ────────────────────────────────────────────────────────────────────
    // 6. SHUTDOWN CHANNEL
    // ────────────────────────────────────────────────────────────────────
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // ────────────────────────────────────────────────────────────────────
    // 6.1 OUTBOX PUBLISHER
    // ────────────────────────────────────────────────────────────────────
    let outbox_repository = Arc::new(OutboxRepository::new(db_pool, "chat-service"));
    let stream_config = OutboxStreamConfig::new("CHAT_EVENTS", vec!["chat.>".to_string()]);
    let outbox_task = OutboxPublisherTask::new(
        "chat-outbox-publisher",
        outbox_repository,
        &nats_url,
        &stream_config,
    )
    .await
    .map_err(|e| {
        BootstrapError::Infrastructure(format!("Failed to start outbox publisher: {e}"))
    })?;
    tokio::spawn(common::infrastructure::background::run_periodic_task(
        outbox_task,
        shutdown_rx.clone(),
    ));
    info!("✅ Outbox publisher task spawned");

    // ────────────────────────────────────────────────────────────────────
    // 7. RUN HTTP + gRPC CONCURRENTLY
    // ────────────────────────────────────────────────────────────────────
    info!(
        "🌐 Starting HTTP server on {}:{}",
        config().service.host,
        config().service.port
    );
    info!(
        "🔗 Starting gRPC server on {}:{}",
        config().service.host,
        config().service.grpc_port.unwrap_or(0)
    );

    let http_shutdown_rx = shutdown_rx.clone();
    let http_handle = tokio::spawn(async move {
        server
            .run(async move {
                let mut rx = http_shutdown_rx;
                let _ = rx.changed().await;
            })
            .await
            .map_err(BootstrapError::Presentation)
    });

    let grpc_addr =
        std::net::SocketAddr::from(([0, 0, 0, 0], config().service.grpc_port.unwrap_or(0)));
    let grpc_shutdown_rx = shutdown_rx.clone();
    let grpc_handle = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .layer(observability::RequestIdScopeLayer)
            .add_service(grpc_router)
            .serve_with_shutdown(grpc_addr, async move {
                let mut rx = grpc_shutdown_rx;
                let _ = rx.changed().await;
            })
            .await
            .map_err(|e| BootstrapError::Infrastructure(format!("gRPC server error: {e}")))
    });

    let mut sig_term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            BootstrapError::Initialization(format!("Failed to setup signal handler: {e}"))
        })?;

    tokio::select! {
        result = http_handle => {
            let res: Result<(), BootstrapError> = result.map_err(|e| {
                BootstrapError::Internal(format!("HTTP server task panicked: {e}"))
            })?;
            res?;
        }
        result = grpc_handle => {
            let res: Result<(), BootstrapError> = result.map_err(|e| {
                BootstrapError::Internal(format!("gRPC server task panicked: {e}"))
            })?;
            res?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, shutting down...");
        }
    }

    let _ = shutdown_tx.send(true);
    Ok(())
}
