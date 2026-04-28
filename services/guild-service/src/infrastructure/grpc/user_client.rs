use common::observability::RequestIdInterceptor;
use std::time::Duration;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::presentation::grpc::proto::user::v1::{
    user_service_client::UserServiceClient, GetUserProfileRequest, UserProfileResponse,
};

/// Wraps the generated tonic user-service client for inter-service calls.
///
/// Uses a lazy connection strategy: the `Endpoint` is configured at construction
/// time but the actual TCP connection is deferred until the first RPC call.
/// This means guild-service starts successfully even if user-service is not
/// yet available.
#[derive(Clone)]
pub struct UserGrpcClient {
    endpoint: Endpoint,
}

impl std::fmt::Debug for UserGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserGrpcClient").finish_non_exhaustive()
    }
}

impl UserGrpcClient {
    /// Configure the gRPC client endpoint without connecting.
    ///
    /// Only validates that `url` is a well-formed URI. The actual TCP handshake
    /// is deferred to the first RPC call via [`connect_lazy`].
    ///
    /// [`connect_lazy`]: tonic::transport::Endpoint::connect_lazy
    pub fn new(url: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let endpoint = Endpoint::from_shared(url.into())?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)));

        Ok(Self { endpoint })
    }

    /// Create a fresh client backed by a lazily-connected channel. The
    /// channel is wrapped with [`RequestIdInterceptor`] so every outbound
    /// call carries the active `x-request-id`.
    fn create_client(
        &self,
    ) -> UserServiceClient<InterceptedService<Channel, RequestIdInterceptor>> {
        let channel = self.endpoint.connect_lazy();
        UserServiceClient::with_interceptor(channel, RequestIdInterceptor)
    }

    /// Get a user profile by ID. Returns `None` if the user does not exist.
    pub async fn get_user_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserProfileResponse>, tonic::Status> {
        let mut client = self.create_client();
        let request = tonic::Request::new(GetUserProfileRequest {
            user_id: user_id.to_string(),
            requester_id: None,
        });

        match client.get_user_profile(request).await {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}
