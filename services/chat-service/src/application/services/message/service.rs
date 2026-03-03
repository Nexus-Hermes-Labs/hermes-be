use std::sync::Arc;
use uuid::Uuid;

use crate::domain::message::{Message, MessageRepository};
use crate::domain::MessageContent;
use crate::infrastructure::events::NatsPublisher;

use super::error::MessageServiceError;

/// Channel message application service.
pub struct MessageService {
    message_repo: Arc<dyn MessageRepository>,
    nats: Arc<NatsPublisher>,
}

impl std::fmt::Debug for MessageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageService").finish_non_exhaustive()
    }
}

impl MessageService {
    pub fn new(message_repo: Arc<dyn MessageRepository>, nats: Arc<NatsPublisher>) -> Self {
        Self { message_repo, nats }
    }

    // ── Send ──────────────────────────────────────────────────────────────

    pub async fn send_message(
        &self,
        channel_id: Uuid,
        user_id: Uuid,
        content: String,
        reply_to_id: Option<Uuid>,
    ) -> Result<Message, MessageServiceError> {
        let content = MessageContent::new(content).map_err(MessageServiceError::DomainError)?;
        let message = Message::new(channel_id, user_id, content, reply_to_id);

        self.message_repo
            .save(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        self.nats.publish_message_created(&message).await;

        Ok(message)
    }

    // ── Get ───────────────────────────────────────────────────────────────

    pub async fn get_messages(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<(Vec<Message>, bool), MessageServiceError> {
        let effective_limit = limit.clamp(1, 100);
        let fetch_limit = effective_limit + 1;

        let mut messages = self
            .message_repo
            .find_by_channel(channel_id, fetch_limit, before_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        let limit_usize = usize::try_from(effective_limit).unwrap_or(100);
        let has_more = messages.len() > limit_usize;
        if has_more {
            messages.truncate(limit_usize);
        }

        Ok((messages, has_more))
    }

    pub async fn get_message(
        &self,
        message_id: Uuid,
        channel_id: Uuid,
    ) -> Result<Message, MessageServiceError> {
        self.message_repo
            .find_by_id_and_channel(message_id, channel_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?
            .ok_or(MessageServiceError::NotFound)
    }

    // ── Edit ──────────────────────────────────────────────────────────────

    pub async fn edit_message(
        &self,
        message_id: Uuid,
        requester_id: Uuid,
        new_content: String,
    ) -> Result<Message, MessageServiceError> {
        let mut message = self
            .message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?
            .ok_or(MessageServiceError::NotFound)?;

        let content = MessageContent::new(new_content).map_err(MessageServiceError::DomainError)?;
        message
            .edit(requester_id, content)
            .map_err(MessageServiceError::DomainError)?;

        self.message_repo
            .update(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        self.nats.publish_message_updated(&message).await;

        Ok(message)
    }

    // ── Delete ────────────────────────────────────────────────────────────

    pub async fn delete_message(
        &self,
        message_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        let mut message = self
            .message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?
            .ok_or(MessageServiceError::NotFound)?;

        message
            .soft_delete(requester_id)
            .map_err(MessageServiceError::DomainError)?;

        self.message_repo
            .update(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        self.nats.publish_message_deleted(&message).await;

        Ok(())
    }

    // ── gRPC helpers ──────────────────────────────────────────────────────

    pub async fn get_messages_grpc(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<(Vec<Message>, bool), MessageServiceError> {
        self.get_messages(channel_id, limit, before_id).await
    }

    pub async fn get_message_grpc(
        &self,
        message_id: Uuid,
    ) -> Result<Message, MessageServiceError> {
        self.message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?
            .ok_or(MessageServiceError::NotFound)
    }

    pub async fn delete_channel_messages_grpc(
        &self,
        channel_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        self.message_repo
            .delete_all_by_channel(channel_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))
    }
}
