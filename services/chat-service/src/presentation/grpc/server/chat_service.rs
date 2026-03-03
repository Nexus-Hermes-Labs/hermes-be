use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::{MessageService, ReactionService};
use crate::presentation::grpc::proto::chat::v1::chat_service_server::ChatService as ChatServiceTrait;
use crate::presentation::grpc::proto::chat::v1::{
    DeleteChannelMessagesRequest, GetChannelMessagesRequest, GetChannelMessagesResponse,
    GetMessageRequest, MessageProto,
};

/// gRPC server implementation for `ChatService`.
#[derive(Debug)]
pub struct ChatServiceGrpc {
    message_service: Arc<MessageService>,
    #[allow(dead_code)]
    reaction_service: Arc<ReactionService>,
}

impl ChatServiceGrpc {
    #[must_use]
    pub const fn new(
        message_service: Arc<MessageService>,
        reaction_service: Arc<ReactionService>,
    ) -> Self {
        Self {
            message_service,
            reaction_service,
        }
    }
}

// ── Converters ───────────────────────────────────────────────────────────────

fn message_to_proto(m: &crate::domain::Message) -> MessageProto {
    MessageProto {
        id: m.id().to_string(),
        channel_id: m.channel_id().to_string(),
        user_id: m.user_id().to_string(),
        content: m.content().as_str().to_string(),
        message_type: m.message_type().as_str().to_string(),
        reply_to_id: m.reply_to_id().map(|id| id.to_string()),
        created_at: Some(prost_types::Timestamp {
            seconds: m.created_at().timestamp(),
            nanos: m.created_at().timestamp_subsec_nanos().cast_signed(),
        }),
        edited_at: m.edited_at().map(|t| prost_types::Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

// ── gRPC Implementation ───────────────────────────────────────────────────────

#[tonic::async_trait]
impl ChatServiceTrait for ChatServiceGrpc {
    async fn get_channel_messages(
        &self,
        request: Request<GetChannelMessagesRequest>,
    ) -> Result<Response<GetChannelMessagesResponse>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;
        let before_id = req
            .before_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid before_id"))?;

        let limit = i64::from(req.limit.clamp(1, 100));

        let (messages, has_more) = self
            .message_service
            .get_messages_grpc(channel_id, limit, before_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetChannelMessagesResponse {
            messages: messages.iter().map(message_to_proto).collect(),
            has_more,
        }))
    }

    async fn get_message(
        &self,
        request: Request<GetMessageRequest>,
    ) -> Result<Response<MessageProto>, Status> {
        let req = request.into_inner();
        let message_id = Uuid::parse_str(&req.message_id)
            .map_err(|_| Status::invalid_argument("Invalid message_id"))?;

        let message = self
            .message_service
            .get_message_grpc(message_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(message_to_proto(&message)))
    }

    async fn delete_channel_messages(
        &self,
        request: Request<DeleteChannelMessagesRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;

        self.message_service
            .delete_channel_messages_grpc(channel_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
}
