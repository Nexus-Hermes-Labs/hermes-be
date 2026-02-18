// src/bootstrap/app_builder.rs
use crate::application::services::authentication::service::AuthService;
use crate::application::services::authentication::user_profile_client::UserProfileClient;
use crate::domain::auth_credential::EmailService;
use crate::infrastructure::grpc::UserGrpcClient; // Changed from LazyUserProfileGrpcClient and LazyUserProfileClientAdapter
use crate::infrastructure::persistence::postgres::{
    PostgresAuthCredentialRepository, PostgresAuthSessionRepository,
};
use crate::infrastructure::security::password::argon2_service::Argon2PasswordService;
use crate::infrastructure::security::token::sha256_service::Sha256TokenHasher;
use crate::presentation::grpc::proto::auth::v1::auth_service_server::AuthServiceServer;
use crate::presentation::grpc::server::AuthServiceGrpc;
use crate::presentation::http::server::Server;
use crate::state::app_state::AppState;
use crate::state::auth_state::AuthState;
use crate::state::shared_state::SharedState;
use common::infrastructure::messaging::{EventPublisher, NatsEventPublisher};
use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::{HealthCheck, Metrics};
use common_config::config;
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

use crate::bootstrap::error::BootstrapError;

// ... (skipping some code)

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
        email_service: Arc<dyn EmailService>,
    ) -> Result<(
        Server,
        AuthServiceServer<AuthServiceGrpc>,
        Arc<PostgresAuthCredentialRepository>,
    ), BootstrapError> {
        let service_name = self.service_name.ok_or_else(|| BootstrapError::Initialization("Service name must be provided".to_string()))?;

        let db_pool = self
            .db_pool
            .clone()
            .ok_or_else(|| BootstrapError::Initialization("Database pool must be provided".to_string()))?;

        let redis = self
            .redis
            .clone()
            .ok_or_else(|| BootstrapError::Initialization("Redis connection must be provided".to_string()))?;

        let metrics = self.metrics.clone().ok_or_else(|| BootstrapError::Initialization("Metrics must be provided".to_string()))?;

        // ========================================
        // INFRASTRUCTURE LAYER
        // ========================================
        let infrastructure = self
            .build_infrastructure(service_name, db_pool, redis, metrics, email_service)
            .await?;

        info!("✅ Infrastructure layer ready");

        // ========================================
        // PERSISTENCE LAYER
        // ========================================
        let repositories = self.build_repositories(&infrastructure)?;
        info!("✅ Persistence layer ready");

        // ========================================
        // DOMAIN LAYER
        // ========================================
        let domain_services = self.build_domain_services()?;
        info!("✅ Domain layer ready");

        // ========================================
        // gRPC CLIENTS (infrastructure adapters)
        // ========================================
        let user_profile_client = Arc::new(
            UserGrpcClient::new(
                config()
                    .grpc_endpoints
                    .user_service
                    .clone()
                    .ok_or_else(|| BootstrapError::Configuration("User service endpoint not configured".to_string()))?,
            )
            .await
            .map_err(|e| BootstrapError::Infrastructure(format!("Failed to create gRPC client: {}", e)))?,
        );

        info!("✅ gRPC clients ready");

        // ========================================
        // APPLICATION LAYER
        // ========================================
        let application = self.build_application(
            repositories,
            domain_services,
            infrastructure.jwt_manager.clone(),
            infrastructure.event_publisher.clone(),
            user_profile_client,
            infrastructure.email_service.clone(),
        )?;

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let app_state = self.compose_state(application.clone(), infrastructure.clone())?;
        info!("✅ Application state composed");

        // ========================================
        // gRPC SERVER
        // ========================================
        let grpc_service = AuthServiceGrpc::new(
            infrastructure.jwt_manager.clone(),
            application.credential_repo.clone(),
            application.session_repo,
        );
        let grpc_router = AuthServiceServer::new(grpc_service);

        info!("✅ gRPC server ready");

        // ========================================
        // HTTP SERVER
        // ========================================
        let server = Server::new(app_state, infrastructure.health_check).await?;
        info!("✅ HTTP server ready");

        Ok((server, grpc_router, application.credential_repo))
    }

    // ========================================
    // PRIVATE BUILDERS
    // ========================================

    async fn build_infrastructure(
        &self,
        service_name: &'static str,
        pool: PgPool,
        redis: redis::aio::ConnectionManager,
        metrics: Metrics,
        email_service: Arc<dyn EmailService>,
    ) -> Result<Infrastructure, BootstrapError> {
        // Health check
        let health_check = Arc::new(HealthCheck::new(pool.clone(), redis.clone()));

        // JWT Manager
        let jwt_manager = Arc::new(
            JwtManager::new(
                &config().secrets.jwt.issuer,
                &config().secrets.jwt.access_secret,
                &config().secrets.jwt.refresh_secret,
            )
            .map_err(|e| BootstrapError::Infrastructure(format!("Failed to create JWT manager: {}", e)))?,
        );

        // Event Publisher
        let event_publisher = Arc::new(
            NatsEventPublisher::new(service_name, &config().nats.get_url())
                .await
                .map_err(|e| BootstrapError::Infrastructure(format!("Failed to create NATS publisher: {}", e)))?,
        );

        Ok(Infrastructure {
            pool,
            redis,
            metrics,
            health_check,
            jwt_manager,
            event_publisher,
            email_service,
        })
    }

    fn build_repositories(&self, infra: &Infrastructure) -> Result<Repositories, BootstrapError> {
        Ok(Repositories {
            credential: Arc::new(PostgresAuthCredentialRepository::new(infra.pool.clone())),
            session: Arc::new(PostgresAuthSessionRepository::new(infra.pool.clone())),
        })
    }

    fn build_domain_services(&self) -> Result<DomainServices, BootstrapError> {
        Ok(DomainServices {
            password: Arc::new(Argon2PasswordService::new()),
            token_hasher: Arc::new(Sha256TokenHasher::new()),
        })
    }

    fn build_application(
        &self,
        repos: Repositories,
        services: DomainServices,
        jwt_manager: Arc<JwtManager>,
        event_publisher: Arc<dyn EventPublisher>,
        user_profile_client: Arc<dyn UserProfileClient>,
        email_service: Arc<dyn EmailService>,
    ) -> Result<Application, BootstrapError> {
        let auth_service = Arc::new(AuthService::new(
            config().service.name.clone(),
            repos.credential.clone(),
            repos.session.clone(),
            services.password,
            services.token_hasher,
            event_publisher,
            jwt_manager,
            user_profile_client,
            email_service,
        ));

        Ok(Application {
            auth_service,
            credential_repo: repos.credential,
            session_repo: repos.session,
        })
    }

    fn compose_state(&self, app: Application, infra: Infrastructure) -> Result<AppState, BootstrapError> {
        let auth_state = AuthState::new(
            app.auth_service,
            infra.jwt_manager,
            app.credential_repo,
            app.session_repo,
        );

        let shared_state = SharedState {
            db: infra.pool,
            redis: infra.redis,
            metrics: infra.metrics,
            email_service: infra.email_service,
        };

        Ok(AppState {
            auth: auth_state,
            shared: shared_state,
        })
    }
}

// ========================================
// INTERNAL TYPES (not exposed outside bootstrap)
// ========================================

#[derive(Clone)]
struct Infrastructure {
    pool: PgPool,
    redis: redis::aio::ConnectionManager,
    metrics: Metrics,
    health_check: Arc<HealthCheck>,
    jwt_manager: Arc<JwtManager>,
    event_publisher: Arc<dyn EventPublisher>,
    email_service: Arc<dyn EmailService>,
}

#[derive(Clone)]
struct Repositories {
    credential: Arc<PostgresAuthCredentialRepository>,
    session: Arc<PostgresAuthSessionRepository>,
}

#[derive(Clone)]
struct DomainServices {
    password: Arc<Argon2PasswordService>,
    token_hasher: Arc<Sha256TokenHasher>,
}

#[derive(Clone)]
struct Application {
    auth_service: Arc<AuthService>,
    credential_repo: Arc<PostgresAuthCredentialRepository>,
    session_repo: Arc<PostgresAuthSessionRepository>,
}
