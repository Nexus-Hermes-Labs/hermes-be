use crate::presentation::http::state::AppState;

use axum::http::{header, HeaderValue, Method};
use common::config::Config;
use common::jwt_manager::JwtManager;
use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::info;
use crate::application::services::user::service::UserApplicationService;
use crate::infrastructure::persistence::postgres::user_repository::repository::PostgresUserRepository;

pub struct Server {
    config: &'static Config,
    pool: PgPool,
    redis: redis::aio::ConnectionManager,
    metrics: Metrics,
}

impl Server {
    pub async fn new(
        config: &'static Config,
        pool: PgPool,
        redis: redis::aio::ConnectionManager,
        metrics: Metrics,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            config,
            pool,
            redis,
            metrics,
        })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        let health_check = Arc::new(HealthCheck::new(self.pool.clone(), self.redis.clone()));

        // Create repositories and services
        let user_repository = Arc::new(PostgresUserRepository::new(self.pool.clone()));

        // Create UserApplicationService with UserService and JwtManager
        let user_service = UserApplicationService::new(Arc::clone(&user_repository));

            let jwt_manager = JwtManager::new(
            &self.config.secrets.jwt.access_secret,
            &self.config.secrets.jwt.refresh_secret,
        );

        // Create unified AppState
        let app_state = AppState::new(self.pool.clone(), user_service, jwt_manager);

        // Build CORS layer
        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_origin(
                self.config
                    .service
                    .allowed_origins
                    .iter()
                    .map(|origin| origin.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>(),
            );

        // Build trace layer
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(LatencyUnit::Millis),
            );

        let version = env!("CARGO_PKG_VERSION");
        info!("Running version {}", version);

        // Create the application router
        let app = super::routes::create_router(app_state, health_check, cors, trace_layer);

        // Create a server address
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.service.port));

        info!("Server listening on {}", addr);

        // Start server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
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

    info!("Shutdown signal received, starting graceful shutdown");
}
