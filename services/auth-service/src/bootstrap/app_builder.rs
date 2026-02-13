// src/bootstrap/app_builder.rs
use crate::application::services::authentication::service::AuthService;
use crate::infrastructure::persistence::postgres::{
    PostgresAuthCredentialRepository, PostgresAuthSessionRepository,
};
use crate::infrastructure::security::password::argon2_service::Argon2PasswordService;
use crate::infrastructure::security::token::sha256_service::Sha256TokenHasher;
use crate::presentation::http::server::Server;
use crate::state::app_state::AppState;
use crate::state::auth_state::AuthState;
use crate::state::shared_state::SharedState;
use anyhow::{Context, Result};
use common::config::config;
use common::infrastructure::messaging::NatsEventPublisher;
use common::infrastructure::security::jwt_manager::JwtManager;
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
    pub async fn build(self) -> Result<Server> {
        let service_name = self.service_name.expect("Service name must be provided");

        let db_pool = self.db_pool.clone().expect("Database pool must be provided");

        let redis = self.redis.clone().expect("Redis connection must be provided");

        let metrics = self.metrics.clone().expect("Metrics must be provided");

        // ========================================
        // INFRASTRUCTURE LAYER
        // ========================================
        let infrastructure = self
            .build_infrastructure(service_name, db_pool, redis, metrics)
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
        // APPLICATION LAYER
        // ========================================
        let application = self.build_application(
            repositories,
            domain_services,
            infrastructure.jwt_manager.clone(),
            infrastructure.event_publisher.clone(),
        )?;

        info!("✅ Application layer ready");

        // ========================================
        // STATE COMPOSITION
        // ========================================
        let app_state = self.compose_state(application, infrastructure.clone())?;
        info!("✅ Application state composed");

        // ========================================
        // SERVER
        // ========================================
        let server = Server::new(app_state, infrastructure.health_check).await?;
        info!("✅ Server ready");

        Ok(server)
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
    ) -> Result<Infrastructure> {
        // Health check
        let health_check = Arc::new(HealthCheck::new(pool.clone(), redis.clone()));

        // JWT Manager
        let jwt_manager = Arc::new(
            JwtManager::new(
                service_name,
                &config().secrets.jwt.access_secret,
                &config().secrets.jwt.refresh_secret,
            )
            .context("Failed to create JWT manager")?,
        );

        // Event Publisher
        let event_publisher = Arc::new(
            NatsEventPublisher::new(service_name, &config().nats.get_url())
                .await
                .context("Failed to create NATS publisher")?,
        );

        Ok(Infrastructure {
            pool,
            redis,
            metrics,
            health_check,
            jwt_manager,
            event_publisher,
        })
    }

    fn build_repositories(&self, infra: &Infrastructure) -> Result<Repositories> {
        Ok(Repositories {
            credential: Arc::new(PostgresAuthCredentialRepository::new(infra.pool.clone())),
            session: Arc::new(PostgresAuthSessionRepository::new(infra.pool.clone())),
        })
    }

    fn build_domain_services(&self) -> Result<DomainServices> {
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
        event_publisher: Arc<NatsEventPublisher>,
    ) -> Result<Application> {
        let auth_service = Arc::new(AuthService::new(
            config().service.name.clone(),
            repos.credential.clone(),
            repos.session.clone(),
            services.password,
            services.token_hasher,
            event_publisher,
            jwt_manager,
        ));

        Ok(Application {
            auth_service,
            credential_repo: repos.credential,
            session_repo: repos.session,
        })
    }

    fn compose_state(&self, app: Application, infra: Infrastructure) -> Result<AppState> {
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
    event_publisher: Arc<NatsEventPublisher>,
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
    auth_service: Arc<
        AuthService<
            PostgresAuthCredentialRepository,
            PostgresAuthSessionRepository,
            Argon2PasswordService,
            Sha256TokenHasher,
            NatsEventPublisher,
        >,
    >,
    credential_repo: Arc<PostgresAuthCredentialRepository>,
    session_repo: Arc<PostgresAuthSessionRepository>,
}
