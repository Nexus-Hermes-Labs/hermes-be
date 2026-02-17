use crate::application::services::{
    UserPrivacyService, UserProfileService, UserRelationshipService,
};
use crate::domain::user_privacy::UserPrivacyRepository;
use crate::domain::user_profile::UserProfileRepository;
use crate::domain::user_relationship::UserRelationshipRepository;
use crate::infrastructure::persistence::postgres::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository, PostgresUserRelationshipRepository,
};
use crate::presentation::grpc::proto::user::v1::user_service_server::UserServiceServer;
use crate::presentation::grpc::server::UserServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::user_state::UserState;
use crate::state::shared_state::SharedState;
use crate::state::AppState;
use crate::bootstrap::error::BootstrapError;
use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Application builder - handles dependency composition
pub struct AppBuilder {
    service_name: Option<&'static str>,
    db_pool: Option<PgPool>,
    redis: Option<redis::aio::ConnectionManager>,
    metrics: Option<Metrics>,
}


impl AppBuilder {
    pub fn new() -> Self {
        Self {
            service_name: None,
            db_pool: None,
            redis: None,
            metrics: None,
        }
    }

    pub fn with_service_name(mut self, service_name: &'static str) -> Self {
        self.service_name = Some(service_name);
        self
    }

    pub fn with_database(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn with_redis(mut self, redis: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Build the complete application with all dependencies
    ///
    /// Returns both the HTTP server and the gRPC service router
    pub async fn build(
        self,
    ) -> Result<(
        Server,
        UserServiceServer<UserServiceGrpc>,
    ), BootstrapError> {
        let _service_name = self.service_name.ok_or_else(|| BootstrapError::Initialization("Service name must be provided".to_string()))?;
        let db_pool = self.db_pool.ok_or_else(|| BootstrapError::Initialization("Database pool must be provided".to_string()))?;
        let redis = self.redis.ok_or_else(|| BootstrapError::Initialization("Redis connection must be provided".to_string()))?;
        let metrics = self.metrics.ok_or_else(|| BootstrapError::Initialization("Metrics must be provided".to_string()))?;

        // ========================================
        // INFRASTRUCTURE LAYER
        // ========================================
        let health_check = Arc::new(HealthCheck::new(db_pool.clone(), redis.clone()));

        info!("✅ Infrastructure layer ready");

        // ========================================
        // PERSISTENCE LAYER
        // ========================================
        let user_profile_repo: Arc<dyn UserProfileRepository> = Arc::new(PostgresUserProfileRepository::new(db_pool.clone()));
        let user_privacy_repo: Arc<dyn UserPrivacyRepository> = Arc::new(PostgresUserPrivacyRepository::new(db_pool.clone()));
        let relationship_repo: Arc<dyn UserRelationshipRepository> = Arc::new(PostgresUserRelationshipRepository::new(db_pool.clone()));

        info!("✅ Persistence layer ready");

        // ========================================
        // APPLICATION LAYER
        // ========================================
        let user_profile_service = Arc::new(UserProfileService::new(user_profile_repo));
        let user_privacy_service = Arc::new(UserPrivacyService::new(user_privacy_repo));
        let relationship_service = Arc::new(UserRelationshipService::new(
            relationship_repo,
            user_privacy_service.clone(),
        ));

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let user_state = UserState::new(
            user_profile_service.clone(),
            user_privacy_service.clone(),
            relationship_service.clone(),
        );

        let shared_state = SharedState {
            db: db_pool,
            redis,
            metrics,
        };

        let app_state = AppState {
            user: user_state,
            shared: shared_state,
        };

        info!("✅ Application state composed");

        // ========================================
        // gRPC SERVER
        // ========================================
        let grpc_service = UserServiceGrpc::new(
            user_profile_service.clone(),
            user_privacy_service.clone(),
            relationship_service,
        );
        let grpc_router = UserServiceServer::new(grpc_service);

        info!("✅ gRPC server ready");

        // ========================================
        // HTTP SERVER
        // ========================================
        let server = Server::new(app_state, health_check).await?;
        info!("✅ HTTP server ready");

        Ok((server, grpc_router))
    }
}
