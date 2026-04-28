mod app_builder;
pub mod error;

use crate::bootstrap::error::BootstrapError;
use common::observability;
use common_config::config;
use tracing::info;

pub use app_builder::AppBuilder;

/// Bootstrap and run the application
pub async fn run(service_name: &'static str) -> Result<(), BootstrapError> {
    // ========================================
    // 1. INITIALIZE METRICS
    // ========================================
    let metrics = observability::metrics::Metrics::init().map_err(|e| {
        BootstrapError::Initialization(format!("Failed to initialize metrics: {e}"))
    })?;

    info!("✅ Metrics initialized");

    // ========================================
    // 2. INITIALIZE DATABASE
    // ========================================
    info!("📦 Connecting to PostgreSQL...");
    let db_pool =
        crate::infrastructure::persistence::postgres::connection::create_pool(&config().database)
            .await
            .map_err(BootstrapError::Database)?;

    info!("✅ Database connected");

    // ========================================
    // 2.1. RUN MIGRATIONS
    // ========================================
    info!("📦 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .map_err(|e| BootstrapError::Initialization(format!("Migration failed: {}", e)))?;
    info!("✅ Migrations applied");

    // ========================================
    // 3. INITIALIZE REDIS
    // ========================================
    info!("🔴 Connecting to Redis...");
    let redis_client =
        redis::Client::open(config().redis.get_url()).map_err(BootstrapError::Redis)?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .map_err(BootstrapError::Redis)?;

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
        .build()?;

    info!("🎯 Application ready!");

    // ========================================
    // 5. INITIALIZE SHUTDOWN SIGNAL
    // ========================================
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // ========================================
    // 6. RUN SERVERS (HTTP + gRPC concurrently)
    // ========================================
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

    // Handle signals for graceful shutdown
    let mut sig_term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            BootstrapError::Initialization(format!("Failed to setup signal handler: {e}"))
        })?;

    tokio::select! {
        result = http_handle => {
            let res: Result<(), BootstrapError> = result.map_err(|e| BootstrapError::Internal(format!("HTTP server task panicked: {e}")))?;
            res?;
        }
        result = grpc_handle => {
            let res: Result<(), BootstrapError> = result.map_err(|e| BootstrapError::Internal(format!("gRPC server task panicked: {e}")))?;
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
