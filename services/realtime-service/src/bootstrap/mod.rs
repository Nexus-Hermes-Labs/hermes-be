//! Service bootstrap — wires all infrastructure and starts HTTP + NATS tasks.

mod app_builder;
pub mod error;

pub use app_builder::AppBuilder;

use crate::bootstrap::error::BootstrapError;
use crate::infrastructure::nats_listener;
use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability;
use common_config::config;
use tracing::info;

/// Bootstrap and run the realtime service.
pub async fn run(service_name: &'static str) -> Result<(), BootstrapError> {
    // ── 1. Metrics ─────────────────────────────────────────────────────────
    let metrics = observability::metrics::Metrics::init().map_err(|e| {
        BootstrapError::Initialization(format!("Failed to initialise metrics: {e}"))
    })?;
    info!("✅ Metrics initialised");

    // ── 2. Redis (kept alive for potential future presence tracking) ────────
    info!("🔴 Connecting to Redis…");
    let redis_url = config().redis.get_url();
    let redis_client = redis::Client::open(redis_url).map_err(BootstrapError::Redis)?;
    let _redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .map_err(BootstrapError::Redis)?;
    info!("✅ Redis connected");

    // ── 3. NATS ────────────────────────────────────────────────────────────
    info!("📨 Connecting to NATS…");
    let nats_url = config().nats.get_url();
    let nats_client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| BootstrapError::Infrastructure(format!("NATS connect error: {e}")))?;
    info!("✅ NATS connected");

    // ── 4. JWT manager ─────────────────────────────────────────────────────
    let jwt_cfg = &config().secrets.jwt;
    let jwt_manager = JwtManager::new(
        &jwt_cfg.issuer,
        &jwt_cfg.access_secret,
        &jwt_cfg.refresh_secret,
    )
    .map_err(BootstrapError::from)?;
    info!("✅ JWT manager ready");

    // ── 5. Build application ───────────────────────────────────────────────
    info!("🔧 Building application…");
    let (server, app_state) = AppBuilder::new()
        .with_service_name(service_name)
        .with_nats(nats_client)
        .with_jwt_manager(jwt_manager)
        .with_metrics(metrics)
        .build()?;
    info!("🎯 Application ready!");

    // ── 6. Shutdown signal ─────────────────────────────────────────────────
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // ── 7. NATS listener task ──────────────────────────────────────────────
    let nats_state = app_state.clone();
    let nats_handle = tokio::spawn(async move {
        nats_listener::run(nats_state).await;
    });

    // ── 8. HTTP server task ────────────────────────────────────────────────
    info!(
        "🌐 Starting HTTP/WS server on {}:{}",
        config().service.host,
        config().service.port
    );

    let http_shutdown_rx = shutdown_rx.clone();
    let http_handle = tokio::spawn(async move {
        server
            .run(async move {
                let mut rx = http_shutdown_rx;
                let _ = rx.changed().await;
            })
            .await
            .map_err(|e| BootstrapError::Presentation(format!("HTTP server error: {e}")))
    });

    // ── 9. Signal handling ─────────────────────────────────────────────────
    let mut sig_term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            BootstrapError::Initialization(format!("Failed to setup signal handler: {e}"))
        })?;

    tokio::select! {
        result = http_handle => {
            let res: Result<(), BootstrapError> = result
                .map_err(|e| BootstrapError::Internal(format!("HTTP task panicked: {e}")))?;
            res?;
        }
        _ = nats_handle => {
            tracing::warn!("NATS listener task exited unexpectedly");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down…");
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, shutting down…");
        }
    }

    let _ = shutdown_tx.send(true);
    Ok(())
}
