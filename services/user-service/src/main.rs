use anyhow::{Context, Result};
use tracing::info;


pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod state;

#[tokio::main]
async fn main() -> Result<()> {
    let service_name = env!("CARGO_PKG_NAME");

    // Initialize configuration
    common_config::init_config(service_name).expect("Failed to load config");

    // Initialize observability
    common::observability::tracing::init_tracing(
        &common_config::config().logging,
        &common_config::config().service.name,
        &common_config::config().service.environment.to_string(),
    )
    .context("Failed to initialize tracing")?;

    info!(
        "🚀 Starting {} v{}",
        common_config::config().service.name,
        common_config::config().service.version
    );
    info!("📍 Environment: {}", common_config::config().service.environment);

    // Bootstrap and run application
    let result = bootstrap::run(service_name).await;

    // Cleanup
    info!("👋 {} stopped", service_name);
    common::observability::tracing::shutdown_tracing();

    result
}
