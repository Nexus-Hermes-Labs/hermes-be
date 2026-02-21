#![allow(missing_docs)]

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let service_name = env!("CARGO_PKG_NAME");

    common_config::init_config(service_name).expect("Failed to load config");

    common::observability::tracing::init_tracing(
        &common_config::config().logging,
        &common_config::config().service.name,
        &common_config::config().service.environment.to_string(),
    ).expect("Failed to init tracing");

    info!("🚀 Starting Channel Service");

    channel_service::bootstrap::run(service_name).await?;

    Ok(())
}
