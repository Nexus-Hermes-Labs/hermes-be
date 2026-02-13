mod app_builder;

use anyhow::{Context, Result};
use common_config::config;
use tracing::info;

pub use app_builder::AppBuilder;

/// Bootstrap and run the application
pub async fn run(_service_name: &'static str) -> Result<()> {
    // ========================================
    // 1. INITIALIZE DATABASE
    // ========================================
    info!("📦 Connecting to PostgreSQL...");
    let _db_pool = crate::infrastructure::persistence::postgres::connection::create_pool(
        &config().database
    )
    .await
    .context("Failed to connect to database")?;

    info!("✅ Database connected");

    // ========================================
    // 2. BUILD APPLICATION
    // ========================================
    info!("🔧 Building application...");

    // let (server, grpc_router) = AppBuilder::new()
    //     .with_service_name(service_name)
    //     .with_database(db_pool)
    //     .build()
    //     .await
    //     .context("Failed to build application")?;

    info!("🎯 Application ready!");

    // ========================================
    // 3. RUN SERVERS (HTTP + gRPC concurrently)
    // ========================================
    // info!(
    //     "🌐 Starting HTTP server on {}:{}",
    //     config().service.host,
    //     config().service.port
    // );
    // info!(
    //     "🔗 Starting gRPC server on {}:{}",
    //     config().service.host,
    //     config().service.grpc_port
    // );

    // let http_handle = tokio::spawn(async move {
    //     server.run().await.context("HTTP server error")
    // });

    // let grpc_addr = std::net::SocketAddr::from(([0, 0, 0, 0], config().service.grpc_port));
    // let grpc_handle = tokio::spawn(async move {
    //     tonic::transport::Server::builder()
    //         .add_service(grpc_router)
    //         .serve(grpc_addr)
    //         .await
    //         .context("gRPC server error")
    // });

    // // Wait for either server to finish (or fail)
    // tokio::select! {
    //     result = http_handle => {
    //         result.context("HTTP server task panicked")??;
    //     }
    //     result = grpc_handle => {
    //         result.context("gRPC server task panicked")??;
    //     }
    // }

    Ok(())
}
