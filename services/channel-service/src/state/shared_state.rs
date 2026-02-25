use crate::infrastructure::grpc::GuildGrpcClient;
use common::observability::Metrics;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared infrastructure state
///
/// Cross-cutting concerns available to all handlers.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SharedState {
    /// `PostgreSQL` connection pool
    pub db: PgPool,
    /// Redis async connection manager
    pub redis: redis::aio::ConnectionManager,
    /// Prometheus metrics
    pub metrics: Metrics,
    /// gRPC client for guild-service authorization checks
    pub guild_grpc_client: Arc<GuildGrpcClient>,
}
