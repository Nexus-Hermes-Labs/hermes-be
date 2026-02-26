//! Messaging Service

use anyhow::{Context, Result};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let service_name = env!("CARGO_PKG_NAME");

    common_config::init_config(service_name).context("Failed to load config")?;

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

    let result = messaging_service::bootstrap::run(service_name)
        .await
        .map_err(anyhow::Error::from);

    info!("👋 {} stopped", service_name);
    common::observability::tracing::shutdown_tracing();

    result
}
