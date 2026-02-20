use crate::application::{GuildInviteService, GuildMemberService, GuildRoleService, GuildService};
use crate::domain::guild::GuildRepository;
use crate::domain::guild_invite::GuildInviteRepository;
use crate::domain::guild_member::GuildMemberRepository;
use crate::domain::guild_role::GuildRoleRepository;
use crate::infrastructure::persistence::{
    PostgresGuildInviteRepository, PostgresGuildMemberRepository, PostgresGuildRepository,
    PostgresGuildRoleRepository,
};
use crate::presentation::grpc::proto::guild::v1::guild_service_server::GuildServiceServer;
use crate::presentation::grpc::server::GuildServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::guild_state::GuildState;
use crate::state::shared_state::SharedState;
use crate::state::AppState;
use crate::bootstrap::error::BootstrapError;
use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::{HealthCheck, Metrics};
use common_config::config;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Builder for composing the guild-service application.
///
/// Collects infrastructure dependencies (database, Redis, metrics) via a fluent
/// API, then wires them together with repositories, services, and HTTP/gRPC servers
/// in [`AppBuilder::build`].
pub struct AppBuilder {
    service_name: Option<&'static str>,
    db_pool: Option<PgPool>,
    redis: Option<redis::aio::ConnectionManager>,
    metrics: Option<Metrics>,
}

impl AppBuilder {
    /// Create a new, unconfigured `AppBuilder`.
    pub fn new() -> Self {
        Self {
            service_name: None,
            db_pool: None,
            redis: None,
            metrics: None,
        }
    }

    /// Set the logical service name (used in tracing and metrics labels).
    pub fn with_service_name(mut self, service_name: &'static str) -> Self {
        self.service_name = Some(service_name);
        self
    }

    /// Provide the PostgreSQL connection pool.
    pub fn with_database(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// Provide the Redis async connection manager.
    pub fn with_redis(mut self, redis: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    /// Provide the Prometheus metrics collector.
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Build the complete application with all dependencies
    pub async fn build(
        self,
    ) -> Result<(Server, GuildServiceServer<GuildServiceGrpc>), BootstrapError> {
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

        let jwt_manager = Arc::new(
            JwtManager::new(
                &config().secrets.jwt.issuer,
                &config().secrets.jwt.access_secret,
                &config().secrets.jwt.refresh_secret,
            )
            .map_err(|e| {
                BootstrapError::Infrastructure(format!("Failed to create JWT manager: {e}"))
            })?,
        );

        info!("✅ Infrastructure layer ready");

        // ========================================
        // PERSISTENCE LAYER
        // ========================================
        let guild_repo: Arc<dyn GuildRepository> =
            Arc::new(PostgresGuildRepository::new(db_pool.clone()));
        let member_repo: Arc<dyn GuildMemberRepository> =
            Arc::new(PostgresGuildMemberRepository::new(db_pool.clone()));
        let role_repo: Arc<dyn GuildRoleRepository> =
            Arc::new(PostgresGuildRoleRepository::new(db_pool.clone()));
        let invite_repo: Arc<dyn GuildInviteRepository> =
            Arc::new(PostgresGuildInviteRepository::new(db_pool.clone()));

        info!("✅ Persistence layer ready");

        // ========================================
        // APPLICATION LAYER
        // ========================================
        let guild_service = Arc::new(GuildService::new(
            guild_repo.clone(),
            member_repo.clone(),
            role_repo.clone(),
        ));
        let member_service = Arc::new(GuildMemberService::new(
            guild_repo.clone(),
            member_repo.clone(),
        ));
        let role_service = Arc::new(GuildRoleService::new(
            guild_repo.clone(),
            role_repo.clone(),
        ));
        let invite_service = Arc::new(GuildInviteService::new(
            guild_repo.clone(),
            invite_repo,
            member_repo,
        ));

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let guild_state = GuildState::new(
            guild_service.clone(),
            member_service.clone(),
            role_service.clone(),
            invite_service.clone(),
        );

        let shared_state = SharedState {
            db: db_pool,
            redis,
            metrics,
            jwt_manager,
        };

        let app_state = AppState {
            guild: guild_state,
            shared: shared_state,
        };

        info!("✅ Application state composed");

        // ========================================
        // gRPC SERVER
        // ========================================
        let grpc_service = GuildServiceGrpc::new(
            guild_service,
            member_service,
            role_service,
            invite_service,
        );
        let grpc_router = GuildServiceServer::new(grpc_service);

        info!("✅ gRPC server ready");

        // ========================================
        // HTTP SERVER
        // ========================================
        let server = Server::new(app_state, health_check).await?;
        info!("✅ HTTP server ready");

        Ok((server, grpc_router))
    }
}
