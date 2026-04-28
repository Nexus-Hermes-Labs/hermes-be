use async_trait::async_trait;
use common::observability::RequestIdInterceptor;
use std::time::Duration;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::application::ports::guild_client::{GuildClient, GuildInfo};
use crate::presentation::grpc::proto::guild::v1::{
    guild_service_client::GuildServiceClient, GetGuildRequest,
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

    /// Create a fresh client backed by a lazily-connected channel. The
    /// channel is wrapped with [`RequestIdInterceptor`] so every outbound
    /// call carries the active `x-request-id`.
    fn create_client(
        &self,
    ) -> GuildServiceClient<InterceptedService<Channel, RequestIdInterceptor>> {
        let channel = self.endpoint.connect_lazy();
        GuildServiceClient::with_interceptor(channel, RequestIdInterceptor)
    }

}

#[async_trait]
impl GuildClient for GuildGrpcClient {
    async fn get_guild(&self, guild_id: Uuid) -> Result<Option<GuildInfo>, String> {
        let mut client = self.create_client();
        let request = tonic::Request::new(GetGuildRequest {
            guild_id: guild_id.to_string(),
            requester_id: None,
        });

        match client.get_guild(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                let owner_id = resp
                    .owner_id
                    .parse::<Uuid>()
                    .map_err(|e| format!("Invalid owner_id from guild-service: {e}"))?;
                Ok(Some(GuildInfo { owner_id }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}
