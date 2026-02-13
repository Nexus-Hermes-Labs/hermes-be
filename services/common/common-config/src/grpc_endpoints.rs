use serde::Deserialize;

/// gRPC endpoints for inter-service communication
///
/// Each service can optionally configure addresses to reach other services via gRPC.
/// Environment variables:
/// - `APP_GRPC_ENDPOINTS__AUTH_SERVICE` (e.g., "http://localhost:50051")
/// - `APP_GRPC_ENDPOINTS__USER_SERVICE` (e.g., "http://localhost:50052")
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GrpcEndpointsConfig {
    #[serde(default = "default_auth_service")]
    pub auth_service: String,
    #[serde(default = "default_user_service")]
    pub user_service: String,
}

fn default_auth_service() -> String {
    "http://localhost:50051".to_string()
}

fn default_user_service() -> String {
    "http://localhost:50052".to_string()
}
