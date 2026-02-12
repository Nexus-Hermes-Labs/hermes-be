use crate::config::logging::LogFormat;
use crate::config::LoggingConfig;
use anyhow::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_tracing(config: &LoggingConfig, service_name: &str, environment: &str) -> Result<()> {
    // Config'ten level al, yoksa RUST_LOG'dan, o da yoksa config'ten
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Format'a göre subscriber kur
    match config.format {
        LogFormat::Json => {
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .json();

            Registry::default().with(env_filter).with(fmt_layer).init();
        }
        LogFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .pretty();

            Registry::default().with(env_filter).with(fmt_layer).init();
        }
    }

    tracing::info!(
        service_name = service_name,
        environment = environment,
        format = ?config.format,
        level = config.level,
        "Tracing initialized"
    );

    Ok(())
}

pub fn shutdown_tracing() {
    tracing::info!("Shutting down tracing");
}
