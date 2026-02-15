use serde::Deserialize;

/// gRPC endpoints for inter-service communication
///
/// Each service can optionally configure addresses to reach other services via gRPC.
/// Environment variables:
/// - `APP_GRPC_ENDPOINTS__AUTH_SERVICE` (e.g., "http://localhost:50051")
/// - `APP_GRPC_ENDPOINTS__USER_SERVICE` (e.g., "http://localhost:50052")
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GrpcEndpointsConfig {
    pub auth_service: Option<String>,
    pub user_service: Option<String>,
}
