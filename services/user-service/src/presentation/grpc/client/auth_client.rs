use crate::presentation::grpc::proto::auth::v1::auth_service_client::AuthServiceClient;
use crate::presentation::grpc::proto::auth::v1::{
    CheckPermissionRequest, CheckPermissionResponse, ValidateTokenRequest, ValidateTokenResponse,
};
use tonic::transport::Channel;
use tracing::{error, info};

/// gRPC client for calling auth-service
///
/// Used by user-service to:
/// - Validate access tokens
/// - Check user permissions
#[derive(Clone)]
pub struct AuthGrpcClient {
    client: AuthServiceClient<Channel>,
}

impl AuthGrpcClient {
    /// Connect to auth-service gRPC server
    pub async fn connect(addr: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let addr = addr.into();
        info!(addr = %addr, "Connecting to auth-service gRPC");

        let client = AuthServiceClient::connect(addr).await?;

        info!("Connected to auth-service gRPC");
        Ok(Self { client })
    }

    /// Validate an access token
    pub async fn validate_token(
        &mut self,
        access_token: &str,
    ) -> Result<ValidateTokenResponse, tonic::Status> {
        let request = tonic::Request::new(ValidateTokenRequest {
            access_token: access_token.to_string(),
        });

        let response = self.client.validate_token(request).await.map_err(|e| {
            error!(error = %e, "Failed to validate token via auth-service gRPC");
            e
        })?;

        Ok(response.into_inner())
    }

    /// Check if a user has a specific permission
    pub async fn check_permission(
        &mut self,
        user_id: &str,
        permission: &str,
        resource_id: &str,
    ) -> Result<CheckPermissionResponse, tonic::Status> {
        let request = tonic::Request::new(CheckPermissionRequest {
            user_id: user_id.to_string(),
            permission: permission.to_string(),
            resource_id: resource_id.to_string(),
        });

        let response = self
            .client
            .check_permission(request)
            .await
            .map_err(|e| {
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
