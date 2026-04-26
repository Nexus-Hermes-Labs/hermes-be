use crate::state::AppState;
use axum::http::{header, HeaderValue, Method};
use common::observability::{request_trace_layer, HealthCheck, HermesTraceLayer};
use common_config::config;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::presentation::error::PresentationError;

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Server {
    app_state: AppState,
    health_check: Arc<HealthCheck>,
}

impl Server {
    #[must_use]
    pub const fn new(app_state: AppState, health_check: Arc<HealthCheck>) -> Self {
        Self {
            app_state,
            health_check,
        }
    }

    pub async fn run(
        self,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), PresentationError> {
        let app = self.build_router();
        let addr = Self::server_address();

        info!("🎧 Server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| PresentationError::HttpServer(e.to_string()))?;

        Ok(())
    }

    fn build_router(&self) -> axum::Router {
        let cors = Self::cors_layer();
        let trace = Self::trace_layer();
        super::routes::create_router(
            self.app_state.clone(),
            self.health_check.clone(),
            cors,
            trace,
        )
    }

    fn cors_layer() -> CorsLayer {
        let origins: Vec<HeaderValue> = config()
            .service
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse::<HeaderValue>().ok())
            .collect();

        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_origin(origins)
    }

    fn trace_layer() -> HermesTraceLayer {
        request_trace_layer()
    }

    fn server_address() -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], config().service.port))
    }
}
