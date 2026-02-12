use anyhow::{Context, Result};
use tracing::info;

use crate::presentation::http::server::Server;
use common::config::{config, Config};
use common::observability;

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
    common::config::init_config(service_name).expect("Failed to load config");

    // Initialize observability
    common::observability::tracing::init_tracing(
        &common::config::config().logging,
        &common::config::config().service.name,
        &common::config::config().service.environment.to_string(),
    )
    .context("Failed to initialize tracing")?;

    info!(
        "🚀 Starting {} v{}",
        common::config::config().service.name,
        common::config::config().service.version
    );
    info!("📍 Environment: {}", common::config::config().service.environment);

    // Bootstrap and run application
    let result = bootstrap::run(service_name).await;

    // Cleanup
    info!("👋 {} stopped", service_name);
    common::observability::tracing::shutdown_tracing();

    result
}