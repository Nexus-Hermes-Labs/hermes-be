//! Axum HTTP server wrapper.

use axum::http::{header, HeaderValue, Method};
use common_config::config;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::state::AppState;

/// Wraps the Axum listener and provides graceful shutdown.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Server {
    app_state: AppState,
}

impl Server {
    /// Create a new server with the given application state.
    #[must_use]
    pub const fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Start listening and serve until `shutdown` resolves.
    pub async fn run(
        self,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> anyhow::Result<()> {
        let cors = Self::cors_layer();
        let router = super::routes::create_router(self.app_state, cors);
        let addr = Self::server_address();

        info!("🎧 Listening on {addr}");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await?;

        Ok(())
    }

    fn cors_layer() -> CorsLayer {
        let origins: Vec<HeaderValue> = config()
            .service
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse::<HeaderValue>().ok())
            .collect();

        CorsLayer::new()
            .allow_methods([Method::GET])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_origin(origins)
    }

    fn server_address() -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], config().service.port))
    }
}
