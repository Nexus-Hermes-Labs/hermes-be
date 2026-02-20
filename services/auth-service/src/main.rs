use anyhow::{Context, Result};
use tracing::info;

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
    info!(
        "📍 Environment: {}",
        common_config::config().service.environment
    );

    // Bootstrap and run application
    let result = auth_service::bootstrap::run(service_name)
        .await
        .map_err(anyhow::Error::from);

    // Cleanup
    info!("👋 {} stopped", service_name);
    common::observability::tracing::shutdown_tracing();

    result
}
