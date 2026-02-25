use crate::application::ChannelService;
use crate::bootstrap::error::BootstrapError;
use crate::domain::channel::ChannelRepository;
use crate::infrastructure::grpc::GuildGrpcClient;
use crate::infrastructure::persistence::postgres::channel::repository::PostgresChannelRepository;
use crate::presentation::grpc::proto::channel::v1::channel_service_server::ChannelServiceServer;
use crate::presentation::grpc::server::ChannelServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::channel_state::ChannelState;
use crate::state::shared_state::SharedState;
use crate::state::AppState;
use common::observability::{HealthCheck, Metrics};
use common_config::config;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Builder for composing the channel-service application.
///
/// Collects infrastructure dependencies (database, Redis, metrics) via a fluent
/// API, then wires them together with repositories, services, and HTTP/gRPC servers
/// in [`AppBuilder::build`].
#[allow(missing_debug_implementations)]
pub struct AppBuilder {
    service_name: Option<&'static str>,
    db_pool: Option<PgPool>,
    redis: Option<redis::aio::ConnectionManager>,
    metrics: Option<Metrics>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    /// Create a new, unconfigured `AppBuilder`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service_name: None,
            db_pool: None,
            redis: None,
            metrics: None,
        }
    }

    /// Set the logical service name (used in tracing and metrics labels).
    #[must_use]
    pub const fn with_service_name(mut self, service_name: &'static str) -> Self {
        self.service_name = Some(service_name);
        self
    }

    /// Provide the `PostgreSQL` connection pool.
    #[must_use]
    pub fn with_database(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// Provide the Redis async connection manager.
    #[must_use]
    pub fn with_redis(mut self, redis: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    /// Provide the Prometheus metrics collector.
    #[must_use]
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Build the complete application with all dependencies.
    pub fn build(
        self,
    ) -> Result<(Server, ChannelServiceServer<ChannelServiceGrpc>), BootstrapError> {
        let _service_name = self.service_name.ok_or_else(|| {
            BootstrapError::Initialization("Service name must be provided".to_string())
        })?;
        let db_pool = self.db_pool.ok_or_else(|| {
            BootstrapError::Initialization("Database pool must be provided".to_string())
        })?;
        let redis = self.redis.ok_or_else(|| {
            BootstrapError::Initialization("Redis connection must be provided".to_string())
        })?;
        let metrics = self.metrics.ok_or_else(|| {
            BootstrapError::Initialization("Metrics must be provided".to_string())
        })?;

        // ========================================
        // INFRASTRUCTURE LAYER
        // ========================================
        let health_check = Arc::new(HealthCheck::new(db_pool.clone(), redis.clone()));

        // Connect to guild-service gRPC for authorization checks
        let guild_grpc_url = config()
            .grpc_endpoints
            .guild_service
            .as_deref()
            .ok_or_else(|| {
                BootstrapError::Configuration(
                    "APP_GRPC_ENDPOINTS__GUILD_SERVICE is not set".to_string(),
                )
            })?
            .to_string();

        let guild_grpc_client = Arc::new(GuildGrpcClient::new(guild_grpc_url).map_err(|e| {
            BootstrapError::Infrastructure(format!("Invalid guild-service gRPC endpoint: {e}"))
        })?);

        info!("✅ Infrastructure layer ready");

        // ========================================
        // PERSISTENCE LAYER
        // ========================================
        let channel_repo: Arc<dyn ChannelRepository> =
            Arc::new(PostgresChannelRepository::new(db_pool.clone()));

        info!("✅ Persistence layer ready");

        // ========================================
        // APPLICATION LAYER
        // ========================================
        let channel_service =
            Arc::new(ChannelService::new(channel_repo, guild_grpc_client.clone()));

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let channel_state = ChannelState::new(channel_service.clone());

        let shared_state = SharedState {
            db: db_pool,
            redis,
            metrics,
            guild_grpc_client,
        };

        let app_state = AppState {
            channel: channel_state,
            shared: shared_state,
        };

        info!("✅ Application state composed");

        // ========================================
        // gRPC SERVER
        // ========================================
        let grpc_service = ChannelServiceGrpc::new(channel_service);
        let grpc_router = ChannelServiceServer::new(grpc_service);

        info!("✅ gRPC server ready");

        // ========================================
        // HTTP SERVER
        // ========================================
        let server = Server::new(app_state, health_check);
        info!("✅ HTTP server ready");

        Ok((server, grpc_router))
    }
}
