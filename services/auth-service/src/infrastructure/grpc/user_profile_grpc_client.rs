use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::application::services::authentication::user_profile_client::{
    UserProfileClient, UserProfileInfo,
};
use crate::infrastructure::grpc::error::UserGrpcError;
use crate::presentation::grpc::proto::user::v1::user_service_client::UserServiceClient;
use crate::presentation::grpc::proto::user::v1::{
    CreateUserProfileRequest, GetUserProfileRequest, UserProfileResponse,
};

/// gRPC client for user-service
#[derive(Clone)]
pub struct UserGrpcClient {
    endpoint: Endpoint,
}

impl UserGrpcClient {
    /// Create new gRPC client
    ///
    /// # Arguments
    /// * `addr` - gRPC server address (e.g., "http://localhost:50052")
    pub async fn new(addr: String) -> Result<Self, tonic::transport::Error> {
        let endpoint = Endpoint::from_shared(addr)?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)));

        Ok(Self { endpoint })
    }

    /// Create a new client instance
    fn create_client(&self) -> UserServiceClient<Channel> {
        let channel = self.endpoint.connect_lazy();
        UserServiceClient::new(channel)
    }
}

impl std::fmt::Debug for UserGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserGrpcClient").finish()
    }
}

// Helper function
fn prost_timestamp_to_chrono(ts: Option<prost_types::Timestamp>) -> DateTime<Utc> {
    match ts {
        Some(t) => DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default(),
        None => Utc::now(),
    }
}

impl TryFrom<UserProfileResponse> for UserProfileInfo {
    type Error = UserGrpcError;

    // TODO: error olusturulduktan sonrasinda format ile heap allocation olusturma!
    fn try_from(res: UserProfileResponse) -> Result<Self, Self::Error> {
        Ok(UserProfileInfo {
            user_id: Uuid::parse_str(&res.user_id)
                .map_err(|e| UserGrpcError::InvalidResponse(format!("Invalid UUID: {}", e)))?,
            username: res.username,
            display_name: res.display_name,
            avatar_url: if res.avatar_url.is_empty() {
                None
            } else {
                Some(res.avatar_url)
            },
            bio: if res.bio.is_empty() {
                None
            } else {
                Some(res.bio)
            },
            created_at: prost_timestamp_to_chrono(res.created_at),
            updated_at: prost_timestamp_to_chrono(res.updated_at),
        })
    }
}

use crate::application::services::authentication::error::AuthApplicationError;

// ... (skipping some code)

#[async_trait]
impl UserProfileClient for UserGrpcClient {
    async fn create_profile(
        &self,
        username: String,
        display_name: String,
        email: String,
    ) -> Result<UserProfileInfo, AuthApplicationError> {
        let mut client = self.create_client();

        let request = tonic::Request::new(CreateUserProfileRequest {
            username,
            display_name,
            email,
            source: "registration".to_string(),
        });

        let response = client
            .create_user_profile(request)
            .await
            .map_err(|e| {
                tracing::error!(
                    code = ?e.code(),
                    message = %e.message(),
                    "gRPC create_user_profile failed"
                );
                match e.code() {
                    tonic::Code::AlreadyExists => {
                        AuthApplicationError::UsernameAlreadyExists(e.message().to_string())
                    }
                    _ => AuthApplicationError::UserServiceError(e.to_string()),
                }
            })?
            .into_inner();

        let info: UserProfileInfo = response.try_into().map_err(|e: UserGrpcError| {
            AuthApplicationError::UserServiceInvalidResponse(e.to_string())
        })?;
        Ok(info)
    }

    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileInfo, AuthApplicationError> {
        let mut client = self.create_client();

        let request = tonic::Request::new(GetUserProfileRequest {
            user_id: user_id.to_string(),
            requester_id: None,
        });

        let response = client
            .get_user_profile(request)
            .await
            .map_err(|e| AuthApplicationError::UserServiceError(e.to_string()))?
            .into_inner();

        let info: UserProfileInfo = response.try_into().map_err(|e: UserGrpcError| {
            AuthApplicationError::UserServiceInvalidResponse(e.to_string())
        })?;
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        // Test endpoint parsing
        let _addr = "http://localhost:50052".to_string();
        // Actual test would be async
    }

    #[test]
    fn test_response_conversion() {
        let response = UserProfileResponse {
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: String::new(),
            bio: String::new(),
            created_at: None,
            updated_at: None,
        };

        let info: Result<UserProfileInfo, _> = response.try_into();
        assert!(info.is_ok());
    }
}
