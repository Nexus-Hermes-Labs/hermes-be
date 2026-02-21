use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::ChannelService;
use crate::presentation::grpc::proto::channel::v1::channel_service_server::ChannelService as ChannelServiceTrait;
use crate::presentation::grpc::proto::channel::v1::{
    ChannelResponse, CreateChannelRequest, DeleteChannelRequest, GetChannelRequest,
    ListGuildChannelsRequest, ListGuildChannelsResponse, UpdateChannelRequest,
};

use crate::domain::Channel;

/// gRPC server implementation for `ChannelService`
#[derive(Debug)]
pub struct ChannelServiceGrpc {
    channel_service: Arc<ChannelService>,
}

impl ChannelServiceGrpc {
    /// Create a new `ChannelServiceGrpc` instance.
    #[must_use]
    pub const fn new(channel_service: Arc<ChannelService>) -> Self {
        Self { channel_service }
    }
}

// ============================================
// CONVERTERS
// ============================================

fn channel_to_proto(c: &Channel) -> ChannelResponse {
    ChannelResponse {
        id: c.id().to_string(),
        guild_id: c.guild_id().to_string(),
        parent_id: c.parent_id().map(|id| id.to_string()),
        name: c.name().as_str().to_string(),
        r#type: match c.channel_type() {
            crate::domain::ChannelType::Text => 0,
            crate::domain::ChannelType::Voice => 1,
            crate::domain::ChannelType::Category => 2,
            crate::domain::ChannelType::Announcement => 3,
        },
        description: c.description().map(String::from),
        position: c.position(),
        created_at: Some(prost_types::Timestamp {
            seconds: c.created_at().timestamp(),
            nanos: c.created_at().timestamp_subsec_nanos().cast_signed(),
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: c.updated_at().timestamp(),
            nanos: c.updated_at().timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

// ============================================
// gRPC IMPLEMENTATION
// ============================================

#[tonic::async_trait]
impl ChannelServiceTrait for ChannelServiceGrpc {
    async fn create_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<ChannelResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;

        // For gRPC, use guild_id as requester for now (internal service call)
        // The actual creator_id would be passed via metadata in production
        let channel_type = match req.r#type {
            0 => "text",
            1 => "voice",
            2 => "category",
            3 => "announcement",
            _ => "text",
        };

        let parent_id = req
            .parent_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid parent_id"))?;

        let channel = self
            .channel_service
            .create_channel(
                guild_id,
                guild_id, // Use guild_id as placeholder — gRPC callers are trusted services
                req.name,
                channel_type.to_string(),
                parent_id,
                req.description,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(channel_to_proto(&channel)))
    }

    async fn get_channel(
        &self,
        request: Request<GetChannelRequest>,
    ) -> Result<Response<ChannelResponse>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;

        let channel = self
            .channel_service
            .get_channel(channel_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(channel_to_proto(&channel)))
    }

    async fn list_guild_channels(
        &self,
        request: Request<ListGuildChannelsRequest>,
    ) -> Result<Response<ListGuildChannelsResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;

        let channels = self
            .channel_service
            .list_guild_channels(guild_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_channels: Vec<ChannelResponse> = channels.iter().map(channel_to_proto).collect();

        Ok(Response::new(ListGuildChannelsResponse {
            channels: proto_channels,
        }))
    }

    async fn update_channel(
        &self,
        request: Request<UpdateChannelRequest>,
    ) -> Result<Response<ChannelResponse>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;

        let parent_id = req
            .parent_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid parent_id"))?;

        // Get channel first to know its guild_id for ownership check
        let channel = self
            .channel_service
            .get_channel(channel_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        let channel = self
            .channel_service
            .update_channel(
                channel_id,
                channel.guild_id(), // Use guild_id as requester (trusted gRPC caller)
                req.name,
                parent_id,
                req.description,
                req.position,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(channel_to_proto(&channel)))
    }

    async fn delete_channel(
        &self,
        request: Request<DeleteChannelRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;

        // Get channel first to know its guild_id for ownership check
        let channel = self
            .channel_service
            .get_channel(channel_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        self.channel_service
            .delete_channel(channel_id, channel.guild_id())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
}
