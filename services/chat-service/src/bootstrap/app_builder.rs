use crate::application::ports::ChatUnitOfWorkFactory;
use crate::application::{MessageService, ReactionService};
use crate::bootstrap::error::BootstrapError;
use crate::domain::message::MessageRepository;
use crate::domain::reaction::ReactionRepository;
use crate::infrastructure::persistence::postgres::PgChatUnitOfWorkFactory;
use crate::infrastructure::persistence::{PostgresMessageRepository, PostgresReactionRepository};
use crate::presentation::grpc::proto::chat::v1::chat_service_server::ChatServiceServer;
use crate::presentation::grpc::server::ChatServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::chat_state::ChatState;
use crate::state::shared_state::SharedState;
use crate::state::AppState;
use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Builder for composing the chat-service application.
#[allow(missing_debug_implementations)]
pub struct AppBuilder {
    service_name: Option<&'static str>,
    db_pool: Option<PgPool>,
    redis: Option<redis::aio::ConnectionManager>,
    metrics: Option<Metrics>,
    nats: Option<async_nats::Client>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service_name: None,
            db_pool: None,
            redis: None,
            metrics: None,
            nats: None,
        }
    }

    #[must_use]
    pub const fn with_service_name(mut self, service_name: &'static str) -> Self {
        self.service_name = Some(service_name);
        self
    }

    #[must_use]
    pub fn with_database(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    #[must_use]
    pub fn with_redis(mut self, redis: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    #[must_use]
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    #[must_use]
    pub fn with_nats(mut self, nats: async_nats::Client) -> Self {
        self.nats = Some(nats);
        self
    }

    /// Build the complete application with all dependencies.
    pub fn build(self) -> Result<(Server, ChatServiceServer<ChatServiceGrpc>), BootstrapError> {
        let service_name = self.service_name.ok_or_else(|| {
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
        let _nats_client = self.nats.ok_or_else(|| {
            BootstrapError::Initialization("NATS client must be provided".to_string())
        })?;

        // ── Infrastructure ────────────────────────────────────────────────
        let health_check = Arc::new(HealthCheck::new(db_pool.clone(), redis.clone()));

        info!("✅ Infrastructure layer ready");

        // ── Persistence ───────────────────────────────────────────────────
        let message_repo: Arc<dyn MessageRepository> =
            Arc::new(PostgresMessageRepository::new(db_pool.clone()));
        let reaction_repo: Arc<dyn ReactionRepository> =
            Arc::new(PostgresReactionRepository::new(db_pool.clone()));
        let uow_factory: Arc<dyn ChatUnitOfWorkFactory> =
            Arc::new(PgChatUnitOfWorkFactory::new(db_pool.clone(), service_name));

        info!("✅ Persistence layer ready");

        // ── Application ───────────────────────────────────────────────────
        let message_service = Arc::new(MessageService::new(
            service_name,
            message_repo.clone(),
            uow_factory.clone(),
        ));
        let reaction_service = Arc::new(ReactionService::new(
            service_name,
            reaction_repo,
            message_repo,
            uow_factory,
        ));

        info!("✅ Application layer ready");

        // ── State ──────────────────────────────────────────────────────────
        let chat_state = ChatState::new(message_service.clone(), reaction_service.clone());
        let shared_state = SharedState {
            db: db_pool,
            redis,
            metrics,
        };
        let app_state = AppState {
            chat: chat_state,
            shared: shared_state,
        };

        info!("✅ Application state composed");

        // ── gRPC ──────────────────────────────────────────────────────────
        let grpc_service = ChatServiceGrpc::new(message_service, reaction_service);
        let grpc_router = ChatServiceServer::new(grpc_service);

        info!("✅ gRPC server ready");

        // ── HTTP ──────────────────────────────────────────────────────────
        let server = Server::new(app_state, health_check);

        info!("✅ HTTP server ready");

        Ok((server, grpc_router))
    }
}
