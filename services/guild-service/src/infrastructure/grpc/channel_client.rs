use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::presentation::grpc::proto::channel::v1::{
    channel_service_client::ChannelServiceClient, CreateDefaultChannelsRequest,
    CreateDefaultChannelsResponse,
};

/// Wraps the generated tonic channel-service client for inter-service calls.
///
/// Uses a lazy connection strategy: the `Endpoint` is configured at construction
/// time but the actual TCP connection is deferred until the first RPC call.
#[derive(Clone)]
pub struct ChannelGrpcClient {
    endpoint: Endpoint,
}

impl std::fmt::Debug for ChannelGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelGrpcClient").finish_non_exhaustive()
    }
}

impl ChannelGrpcClient {
    /// Configure the gRPC client endpoint without connecting.
    pub fn new(url: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let endpoint = Endpoint::from_shared(url.into())?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)));

        Ok(Self { endpoint })
    }

    fn create_client(&self) -> ChannelServiceClient<Channel> {
        let channel = self.endpoint.connect_lazy();
        ChannelServiceClient::new(channel)
    }

    /// Create default `general` (text) and `General` (voice) channels for a new guild.
    pub async fn create_default_channels(
        &self,
        guild_id: Uuid,
    ) -> Result<CreateDefaultChannelsResponse, tonic::Status> {
        let mut client = self.create_client();
        let request = tonic::Request::new(CreateDefaultChannelsRequest {
            guild_id: guild_id.to_string(),
        });
        let response = client.create_default_channels(request).await?;
        Ok(response.into_inner())
    }
}
