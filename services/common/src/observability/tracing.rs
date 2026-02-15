use anyhow::Result;
use common_config::logging::LogFormat;
use common_config::LoggingConfig;
use tracing_appender::rolling;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
    Registry,
};

pub fn init_tracing(config: &LoggingConfig, service_name: &str, environment: &str) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    match config.format {
        LogFormat::Json => {
            let console_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .json();
            let file_appender = rolling::daily("logs", "error.log");
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .json()
                .with_filter(LevelFilter::ERROR);
            Registry::default()
                .with(env_filter)
                .with(console_layer)
                .with(file_layer)
                .init();
        }
        LogFormat::Pretty => {
            let console_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .pretty()
                .with_ansi(true);
            let file_appender = rolling::daily("logs", "error.log");
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .pretty()
                .with_filter(LevelFilter::ERROR);
            Registry::default()
                .with(env_filter)
                .with(console_layer)
                .with(file_layer)
                .init();
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
