use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tonic::transport::Channel;
use tracing::{error, info};
use uuid::Uuid;

use crate::application::services::authentication::user_profile_client::{
    UserProfileClient, UserProfileInfo,
};
use crate::presentation::grpc::proto::user::v1::user_service_client::UserServiceClient;
use crate::presentation::grpc::proto::user::v1::{
    CreateUserProfileRequest, GetUserProfileRequest,
};

#[derive(Debug, Error)]
pub enum UserProfileGrpcError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC call failed: {0}")]
    Status(#[from] tonic::Status),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// gRPC-based implementation of `UserProfileClient`
///
/// Calls user-service via gRPC to manage user profiles.
/// Lives in the infrastructure layer as it's an external adapter.
#[derive(Clone)]
pub struct UserProfileGrpcClient {
    client: UserServiceClient<Channel>,
}

impl UserProfileGrpcClient {
    /// Connect to user-service gRPC server
    pub async fn connect(addr: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let addr = addr.into();
        info!(addr = %addr, "Connecting to user-service gRPC");

        let client = UserServiceClient::connect(addr).await?;

        info!("Connected to user-service gRPC");
        Ok(Self { client })
    }
}

impl std::fmt::Debug for UserProfileGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserProfileGrpcClient").finish()
    }
}

/// Convert a proto Timestamp to chrono DateTime<Utc>
fn timestamp_to_datetime(ts: &prost_types::Timestamp) -> DateTime<Utc> {
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_default()
}

#[async_trait]
impl UserProfileClient for UserProfileGrpcClient {
    type Error = UserProfileGrpcError;

    async fn create_profile(
        &self,
        user_id: Uuid,
        username: &str,
        display_name: &str,
        email: &str,
    ) -> Result<UserProfileInfo, Self::Error> {
        let request = tonic::Request::new(CreateUserProfileRequest {
            user_id: user_id.to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            source: "registration".to_string(),
        });

        let response = self
            .client
            .clone()
            .create_user_profile(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to create user profile via user-service gRPC");
                e
            })?;

        let profile = response.into_inner();

        Ok(UserProfileInfo {
            user_id: Uuid::parse_str(&profile.user_id).map_err(|e| {
                UserProfileGrpcError::InvalidResponse(format!("Invalid user_id: {e}"))
            })?,
            username: profile.username,
            discriminator: profile.discriminator,
            display_name: profile.display_name,
            avatar_url: if profile.avatar_url.is_empty() {
                None
            } else {
                Some(profile.avatar_url)
            },
            bio: if profile.bio.is_empty() {
                None
            } else {
                Some(profile.bio)
            },
            created_at: profile
                .created_at
                .as_ref()
                .map(timestamp_to_datetime)
                .unwrap_or_default(),
            updated_at: profile
                .updated_at
                .as_ref()
                .map(timestamp_to_datetime)
                .unwrap_or_default(),
        })
    }

    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileInfo, Self::Error> {
        let request = tonic::Request::new(GetUserProfileRequest {
            user_id: user_id.to_string(),
        });

        let response = self
            .client
            .clone()
            .get_user_profile(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to get user profile via user-service gRPC");
                e
            })?;

        let profile = response.into_inner();

        Ok(UserProfileInfo {
            user_id: Uuid::parse_str(&profile.user_id).map_err(|e| {
                UserProfileGrpcError::InvalidResponse(format!("Invalid user_id: {e}"))
            })?,
            username: profile.username,
            discriminator: profile.discriminator,
            display_name: profile.display_name,
            avatar_url: if profile.avatar_url.is_empty() {
                None
            } else {
                Some(profile.avatar_url)
            },
            bio: if profile.bio.is_empty() {
                None
            } else {
                Some(profile.bio)
            },
            created_at: profile
                .created_at
                .as_ref()
                .map(timestamp_to_datetime)
                .unwrap_or_default(),
            updated_at: profile
                .updated_at
                .as_ref()
                .map(timestamp_to_datetime)
                .unwrap_or_default(),
        })
    }
}
