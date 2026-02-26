use std::sync::Arc;

use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use tracing::info;

use crate::application::{ConversationService, MessageService, ReactionService};
use crate::bootstrap::error::BootstrapError;
use crate::domain::conversation::ConversationRepository;
use crate::domain::message::MessageRepository;
use crate::domain::reaction::ReactionRepository;
use crate::infrastructure::persistence::{
    PostgresConversationRepository, PostgresMessageRepository, PostgresReactionRepository,
};
use crate::infrastructure::NatsPublisher;
use crate::presentation::grpc::proto::messaging::v1::messaging_service_server::MessagingServiceServer;
use crate::presentation::grpc::server::MessagingServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::messaging_state::MessagingState;
use crate::state::shared_state::SharedState;
use crate::state::AppState;

/// Builder for composing the messaging-service application.
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

    pub fn build(
        self,
    ) -> Result<(Server, MessagingServiceServer<MessagingServiceGrpc>), BootstrapError> {
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
        let nats_client = self.nats.ok_or_else(|| {
            BootstrapError::Initialization("NATS client must be provided".to_string())
        })?;

        // ========================================
        // INFRASTRUCTURE LAYER
        // ========================================
        let health_check = Arc::new(HealthCheck::new(db_pool.clone(), redis.clone()));
        let nats_publisher = Arc::new(NatsPublisher::new(nats_client.clone()));

        info!("✅ Infrastructure layer ready");

        // ========================================
        // PERSISTENCE LAYER
        // ========================================
        let message_repo: Arc<dyn MessageRepository> =
            Arc::new(PostgresMessageRepository::new(db_pool.clone()));
        let reaction_repo: Arc<dyn ReactionRepository> =
            Arc::new(PostgresReactionRepository::new(db_pool.clone()));
        let conversation_repo: Arc<dyn ConversationRepository> =
            Arc::new(PostgresConversationRepository::new(db_pool.clone()));

        info!("✅ Persistence layer ready");

        // ========================================
        // APPLICATION LAYER
        // ========================================
        let message_service = Arc::new(MessageService::new(message_repo, nats_publisher.clone()));
        let reaction_service =
            Arc::new(ReactionService::new(reaction_repo, nats_publisher.clone()));
        let conversation_service =
            Arc::new(ConversationService::new(conversation_repo, nats_publisher));

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let messaging_state = MessagingState::new(
            message_service.clone(),
            reaction_service.clone(),
            conversation_service.clone(),
        );
        let shared_state = SharedState {
            db: db_pool,
            redis,
            metrics,
            nats: nats_client,
        };
        let app_state = AppState {
            messaging: messaging_state,
            shared: shared_state,
        };

        info!("✅ Application state composed");

        // ========================================
        // gRPC SERVER
        // ========================================
        let grpc_impl =
            MessagingServiceGrpc::new(message_service, reaction_service, conversation_service);
        let grpc_router = MessagingServiceServer::new(grpc_impl);

        info!("✅ gRPC server ready");

        // ========================================
        // HTTP SERVER
        // ========================================
        let server = Server::new(app_state, health_check);
        info!("✅ HTTP server ready");

        Ok((server, grpc_router))
    }
}
