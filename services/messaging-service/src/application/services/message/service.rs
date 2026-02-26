use std::sync::Arc;
use uuid::Uuid;

use crate::domain::message::{Message, MessageContent, MessageRepository, MessageTarget};
use crate::infrastructure::NatsPublisher;

use super::error::MessageServiceError;

pub struct MessageService {
    message_repo: Arc<dyn MessageRepository>,
    nats:         Arc<NatsPublisher>,
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

    // ── Queries ───────────────────────────────────────────────────────────

    pub async fn get_channel_messages(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, MessageServiceError> {
        self.message_repo
            .find_by_channel(channel_id, limit.clamp(1, 100), before_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))
    }

    pub async fn get_conversation_messages(
        &self,
        conversation_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, MessageServiceError> {
        self.message_repo
            .find_by_conversation(conversation_id, limit.clamp(1, 100), before_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))
    }

    pub async fn get_message(&self, message_id: Uuid) -> Result<Message, MessageServiceError> {
        self.message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?
            .ok_or(MessageServiceError::NotFound)
    }

    // ── Commands ──────────────────────────────────────────────────────────

    pub async fn send_channel_message(
        &self,
        channel_id: Uuid,
        user_id: Uuid,
        content: String,
        reply_to_id: Option<Uuid>,
    ) -> Result<Message, MessageServiceError> {
        let content_vo = MessageContent::new(content)?;
        let message = Message::new(
            MessageTarget::Channel(channel_id),
            user_id,
            content_vo,
            reply_to_id,
        );
        self.message_repo
            .save(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;
        self.nats.publish_message_created(&message).await;
        Ok(message)
    }

    pub async fn send_conversation_message(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        content: String,
        reply_to_id: Option<Uuid>,
    ) -> Result<Message, MessageServiceError> {
        let content_vo = MessageContent::new(content)?;
        let message = Message::new(
            MessageTarget::Conversation(conversation_id),
            user_id,
            content_vo,
            reply_to_id,
        );
        self.message_repo
            .save(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;
        self.nats.publish_message_created(&message).await;
        Ok(message)
    }

    pub async fn edit_message(
        &self,
        message_id: Uuid,
        requester_id: Uuid,
        new_content: String,
    ) -> Result<Message, MessageServiceError> {
        let mut message = self.get_message(message_id).await?;
        let content_vo = MessageContent::new(new_content)?;
        message.edit(requester_id, content_vo)?;
        self.message_repo
            .update(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;
        self.nats.publish_message_updated(&message).await;
        Ok(message)
    }

    pub async fn delete_message(
        &self,
        message_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        let mut message = self.get_message(message_id).await?;
        message.soft_delete(requester_id)?;
        self.message_repo
            .update(&message)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;
        self.nats.publish_message_deleted(&message).await;
        Ok(())
    }

    pub async fn delete_channel_messages(&self, channel_id: Uuid) -> Result<(), MessageServiceError> {
        self.message_repo
            .delete_all_by_channel(channel_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))
    }
}
