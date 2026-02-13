use crate::presentation::grpc::proto::user::v1::user_service_client::UserServiceClient;
use crate::presentation::grpc::proto::user::v1::{
    CreateUserProfileRequest, GetUserProfileRequest, UserProfileResponse,
};
use tonic::transport::Channel;
use tracing::{error, info};

/// gRPC client for calling user-service
///
/// Used by auth-service to:
/// - Create user profiles during registration
/// - Get user profile information
#[derive(Clone)]
pub struct UserGrpcClient {
    client: UserServiceClient<Channel>,
}

impl UserGrpcClient {
    /// Connect to user-service gRPC server
    pub async fn connect(addr: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let addr = addr.into();
        info!(addr = %addr, "Connecting to user-service gRPC");

        let client = UserServiceClient::connect(addr).await?;

        info!("Connected to user-service gRPC");
        Ok(Self { client })
    }

    /// Create a user profile (called during registration)
    pub async fn create_user_profile(
        &mut self,
        user_id: &str,
        username: &str,
        display_name: &str,
        email: &str,
        source: &str,
    ) -> Result<UserProfileResponse, tonic::Status> {
        let request = tonic::Request::new(CreateUserProfileRequest {
            user_id: user_id.to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            source: source.to_string(),
        });

        let response = self
            .client
            .create_user_profile(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to create user profile via user-service gRPC");
                e
            })?;

        Ok(response.into_inner())
    }

    /// Get a user profile by ID
    pub async fn get_user_profile(
        &mut self,
        user_id: &str,
    ) -> Result<UserProfileResponse, tonic::Status> {
        let request = tonic::Request::new(GetUserProfileRequest {
            user_id: user_id.to_string(),
        });

        let response = self
            .client
            .get_user_profile(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to get user profile via user-service gRPC");
                e
            })?;

        Ok(response.into_inner())
    }
}

impl std::fmt::Debug for UserGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserGrpcClient").finish()
    }
}
