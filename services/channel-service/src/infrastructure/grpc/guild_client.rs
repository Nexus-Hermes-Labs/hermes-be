use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::presentation::grpc::proto::guild::v1::{
    guild_service_client::GuildServiceClient, GetGuildRequest, GuildResponse,
};

/// Wraps the generated tonic guild-service client for inter-service authorization.
///
/// Uses a lazy connection strategy: the `Endpoint` is configured at construction
/// time but the actual TCP connection is deferred until the first RPC call.
/// This means channel-service starts successfully even if guild-service is not
/// yet available.
#[derive(Clone)]
pub struct GuildGrpcClient {
    endpoint: Endpoint,
}

impl std::fmt::Debug for GuildGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuildGrpcClient").finish_non_exhaustive()
    }
}

impl GuildGrpcClient {
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

    /// Create a fresh client backed by a lazily-connected channel.
    fn create_client(&self) -> GuildServiceClient<Channel> {
        let channel = self.endpoint.connect_lazy();
        GuildServiceClient::new(channel)
    }

    /// Get a guild by ID. Returns `None` if the guild does not exist.
    pub async fn get_guild(&self, guild_id: Uuid) -> Result<Option<GuildResponse>, tonic::Status> {
        let mut client = self.create_client();
        let request = tonic::Request::new(GetGuildRequest {
            guild_id: guild_id.to_string(),
            requester_id: None,
        });

        match client.get_guild(request).await {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}
