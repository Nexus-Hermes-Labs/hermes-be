use crate::state::AppState;
use axum::http::{header, HeaderValue, Method};
use common::config::config;
use common::observability::HealthCheck;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::info;

#[derive(Clone)]
pub struct Server {
    app_state: AppState,
    health_check: Arc<HealthCheck>,
}

impl Server {
    pub async fn new(
        app_state: AppState,
        health_check: Arc<HealthCheck>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            app_state,
            health_check,
        })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        let app = self.build_router();
        let addr = self.server_address();

        info!("🎧 Server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }

    fn build_router(&self) -> axum::Router {
        let cors = self.cors_layer();
        let trace = self.trace_layer();

        super::routes::create_router(
            self.app_state.clone(),
            self.health_check.clone(),
            cors,
            trace,
        )
    }

    fn cors_layer(&self) -> CorsLayer {
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_origin(
                config()
                    .service
                    .allowed_origins
                    .iter()
                    .map(|origin| origin.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>(),
            )
    }

    fn trace_layer(
        &self,
    ) -> TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    > {
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(LatencyUnit::Millis),
            )
    }

    fn server_address(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], config().service.port))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("⏹️  Shutdown signal received, starting graceful shutdown");
}
