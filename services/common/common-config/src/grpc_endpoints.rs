use serde::Deserialize;

/// gRPC endpoints for inter-service communication
///
/// Each service can optionally configure addresses to reach other services via gRPC.
/// Environment variables:
/// - `APP_GRPC_ENDPOINTS__AUTH_SERVICE` (e.g., "http://localhost:50051")
/// - `APP_GRPC_ENDPOINTS__USER_SERVICE` (e.g., "http://localhost:50052")
/// - `APP_GRPC_ENDPOINTS__GUILD_SERVICE` (e.g., "http://localhost:50056")
/// - `APP_GRPC_ENDPOINTS__CHANNEL_SERVICE` (e.g., "http://localhost:50053")
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct GrpcEndpointsConfig {
    pub auth_service: Option<String>,
    pub user_service: Option<String>,
    pub guild_service: Option<String>,
    pub channel_service: Option<String>,
}
