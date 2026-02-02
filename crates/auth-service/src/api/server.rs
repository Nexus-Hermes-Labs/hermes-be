use crate::api::state::AppState;
use crate::infrastructure::persistence::postgres::user_repository::PostgresAuthUserRepository;
use axum::http::{header, HeaderValue, Method};
use common::config::Config;
use common::jwt::JwtManager;
use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::info;
use crate::application::services::auth::service::AuthService;

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

        // Create a JWT manager
        let jwt_manager = JwtManager::new(
            &self.config.secrets.jwt.access_secret,
            &self.config.secrets.jwt.refresh_secret,
        );

        // Create repositories and services
        let user_repository = Arc::new(PostgresAuthUserRepository::new(self.pool.clone()));

        // Create password service
        let argon2_password_service = Arc::new(
            crate::infrastructure::security::argon2_password_service::Argon2PasswordService::new(),
        );

        // Create AuthService with UserService and JwtManager
        let auth_service = AuthService::new(
            Arc::clone(&user_repository),
            argon2_password_service,
            jwt_manager.clone(),
        );

        // Create unified AppState
        let app_state = AppState::new(self.pool.clone(), auth_service, jwt_manager);

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
