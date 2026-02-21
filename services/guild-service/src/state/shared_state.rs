use crate::infrastructure::grpc::UserGrpcClient;
use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::Metrics;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared infrastructure state injected into every HTTP handler and gRPC service.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SharedState {
    /// `PostgreSQL` connection pool for all database operations.
    pub db: PgPool,
    /// Redis async connection manager used for caching and session storage.
    pub redis: redis::aio::ConnectionManager,
    /// Prometheus metrics collector for observability.
    pub metrics: Metrics,
    /// JWT manager for issuing and verifying access/refresh tokens.
    pub jwt_manager: Arc<JwtManager>,
    /// Lazy gRPC client for user-service (profile lookups, existence checks).
    pub user_grpc_client: Arc<UserGrpcClient>,
}
