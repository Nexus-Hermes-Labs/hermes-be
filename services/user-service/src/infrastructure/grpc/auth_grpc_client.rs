use crate::presentation::grpc::proto::auth::v1::auth_service_client::AuthServiceClient;
use crate::presentation::grpc::proto::auth::v1::{
    CheckPermissionRequest, CheckPermissionResponse, ValidateTokenRequest, ValidateTokenResponse,
};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tracing::{error, info};

/// gRPC client for calling auth-service
///
/// Used by user-service to:
/// - Validate access tokens
/// - Check user permissions
///
/// Lives in the infrastructure layer as it's an external adapter.
#[derive(Clone)]
pub struct AuthGrpcClient {
    endpoint: Endpoint,  // ✅ Endpoint store et, Channel değil
}

impl AuthGrpcClient {
    /// Connect to auth-service gRPC server
    pub async fn connect(addr: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let addr = addr.into();
        info!(addr = %addr, "Connecting to auth-service gRPC");

        let endpoint = Endpoint::from_shared(addr)?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)));

        info!("Auth-service gRPC endpoint configured");
        Ok(Self { endpoint })
    }

    /// Create a new client instance
    fn create_client(&self) -> AuthServiceClient<Channel> {
        let channel = self.endpoint.connect_lazy();
        AuthServiceClient::new(channel)
    }

    /// Validate an access token
    pub async fn validate_token(
        &self,  // ✅ &mut yerine &self
        access_token: &str,
    ) -> Result<ValidateTokenResponse, tonic::Status> {
        let mut client = self.create_client();

        let request = tonic::Request::new(ValidateTokenRequest {
            access_token: access_token.to_string(),
        });

        let response = client.validate_token(request).await.map_err(|e| {
            error!(error = %e, "Failed to validate token via auth-service gRPC");
            e
        })?;

        Ok(response.into_inner())
    }

    /// Check if a user has a specific permission
    pub async fn check_permission(
        &self,  // ✅ &mut yerine &self
        user_id: &str,
        permission: &str,
        resource_id: &str,
    ) -> Result<CheckPermissionResponse, tonic::Status> {
        let mut client = self.create_client();

        let request = tonic::Request::new(CheckPermissionRequest {
            user_id: user_id.to_string(),
            permission: permission.to_string(),
            resource_id: resource_id.to_string(),
        });

        let response = client.check_permission(request).await.map_err(|e| {
            error!(error = %e, "Failed to check permission via auth-service gRPC");
            e
        })?;

        Ok(response.into_inner())
    }
}

impl std::fmt::Debug for AuthGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthGrpcClient").finish()
    }
}