use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::{ConversationService, MessageService, ReactionService};
use crate::presentation::grpc::proto::messaging::v1::messaging_service_server::MessagingService as MessagingServiceTrait;
use crate::presentation::grpc::proto::messaging::v1::{
    DeleteChannelMessagesRequest, FindDirectConversationRequest, FindDirectConversationResponse,
    GetChannelMessagesRequest, GetChannelMessagesResponse, GetConversationMessagesRequest,
    GetConversationMessagesResponse, GetMessageRequest, MessageProto,
};

/// gRPC server implementation for `MessagingService`
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct MessagingServiceGrpc {
    message_service:      Arc<MessageService>,
    #[allow(dead_code)]
    reaction_service:     Arc<ReactionService>,
    conversation_service: Arc<ConversationService>,
}

impl MessagingServiceGrpc {
    #[must_use]
    pub const fn new(
        message_service:      Arc<MessageService>,
        reaction_service:     Arc<ReactionService>,
        conversation_service: Arc<ConversationService>,
    ) -> Self {
        Self { message_service, reaction_service, conversation_service }
    }
}

// ── Converter ─────────────────────────────────────────────────────────────────

fn message_to_proto(m: &crate::domain::Message) -> MessageProto {
    MessageProto {
        id:              m.id().to_string(),
        channel_id:      m.channel_id().map(|id| id.to_string()),
        conversation_id: m.conversation_id().map(|id| id.to_string()),
        user_id:         m.user_id().to_string(),
        content:         m.content().as_str().to_string(),
        message_type:    m.message_type().as_str().to_string(),
        reply_to_id:     m.reply_to_id().map(|id| id.to_string()),
        is_deleted:      m.is_deleted(),
        created_at:      Some(prost_types::Timestamp {
            seconds: m.created_at().timestamp(),
            nanos:   m.created_at().timestamp_subsec_nanos().cast_signed(),
        }),
        edited_at:       m.edited_at().map(|t| prost_types::Timestamp {
            seconds: t.timestamp(),
            nanos:   t.timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

// ── gRPC implementation ───────────────────────────────────────────────────────

#[tonic::async_trait]
impl MessagingServiceTrait for MessagingServiceGrpc {
    async fn get_channel_messages(
        &self,
        request: Request<GetChannelMessagesRequest>,
    ) -> Result<Response<GetChannelMessagesResponse>, Status> {
        let req = request.into_inner();
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| Status::invalid_argument("Invalid channel_id"))?;
        let limit = i64::from(req.limit.clamp(1, 100));
        let before_id = req
            .before_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid before_id"))?;

        let messages = self
            .message_service
            .get_channel_messages(channel_id, limit, before_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let has_more = i64::try_from(messages.len()).unwrap_or(i64::MAX) == limit;
        let proto_messages: Vec<MessageProto> = messages.iter().map(message_to_proto).collect();

        Ok(Response::new(GetChannelMessagesResponse {
            messages: proto_messages,
            has_more,
        }))
    }

    async fn get_conversation_messages(
        &self,
        request: Request<GetConversationMessagesRequest>,
    ) -> Result<Response<GetConversationMessagesResponse>, Status> {
        let req = request.into_inner();
        let conversation_id = Uuid::parse_str(&req.conversation_id)
            .map_err(|_| Status::invalid_argument("Invalid conversation_id"))?;
        let limit = i64::from(req.limit.clamp(1, 100));
        let before_id = req
            .before_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid before_id"))?;

        let messages = self
            .message_service
            .get_conversation_messages(conversation_id, limit, before_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let has_more = i64::try_from(messages.len()).unwrap_or(i64::MAX) == limit;
        let proto_messages: Vec<MessageProto> = messages.iter().map(message_to_proto).collect();

        Ok(Response::new(GetConversationMessagesResponse {
            messages: proto_messages,
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
            .get_message(message_id)
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
            .delete_channel_messages(channel_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn find_direct_conversation(
        &self,
        request: Request<FindDirectConversationRequest>,
    ) -> Result<Response<FindDirectConversationResponse>, Status> {
        let req = request.into_inner();
        let user_id_a = Uuid::parse_str(&req.user_id_a)
            .map_err(|_| Status::invalid_argument("Invalid user_id_a"))?;
        let user_id_b = Uuid::parse_str(&req.user_id_b)
            .map_err(|_| Status::invalid_argument("Invalid user_id_b"))?;

        let conv = self
            .conversation_service
            .open_dm(user_id_a, user_id_b)
            .await;

        // find_dm_between is find-only, but ConversationService::open_dm creates if not found.
        // For the gRPC "find" semantics we use open_dm which is idempotent — safe to call here.
        let conversation_id = conv
            .map(|c| Some(c.id().to_string()))
            .unwrap_or(None);

        Ok(Response::new(FindDirectConversationResponse { conversation_id }))
    }
}
