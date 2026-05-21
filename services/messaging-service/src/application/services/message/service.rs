use std::sync::Arc;
use uuid::Uuid;

use common::domain::event::IntoEventEnvelope;
use common::infrastructure::outbox::NewOutboxEvent;

use crate::application::events::{
    MessagingMessageCreatedEvent, MessagingMessageDeletedEvent, MessagingMessageUpdatedEvent,
    AGGREGATE_TYPE_MESSAGE,
};
use crate::application::ports::unit_of_work::MessagingUnitOfWorkFactory;
use crate::domain::message::{Message, MessageContent, MessageRepository, MessageTarget};

use super::error::MessageServiceError;

pub struct MessageService {
    service_name: String,
    message_repo: Arc<dyn MessageRepository>,
    uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
}

impl std::fmt::Debug for MessageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageService").finish_non_exhaustive()
    }
}

impl MessageService {
    pub fn new(
        service_name: impl Into<String>,
        message_repo: Arc<dyn MessageRepository>,
        uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            message_repo,
            uow_factory,
        }
    }

    fn outbox_event(
        &self,
        aggregate_id: Uuid,
        event: impl IntoEventEnvelope,
    ) -> Result<NewOutboxEvent, MessageServiceError> {
        let event_type = event.event_type().to_string();
        let envelope = event.into_envelope(&self.service_name);
        let payload = serde_json::to_value(&envelope)
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;
        Ok(NewOutboxEvent {
            id: envelope.event_id,
            aggregate_id,
            aggregate_type: AGGREGATE_TYPE_MESSAGE.to_string(),
            event_type,
            payload,
        })
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

    async fn send_message(&self, target: MessageTarget, user_id: Uuid, content: String, reply_to_id: Option<Uuid>) -> Result<Message, MessageServiceError> {
        let content_vo = MessageContent::new(content)?;
        let message = Message::new(target, user_id, content_vo, reply_to_id);

        let outbox = self.outbox_event(
            message.id(),
            MessagingMessageCreatedEvent::from_message(&message),
        )?;
        let message_for_tx = message.clone();
        let outbox_for_tx = outbox.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.messages().save(&message_for_tx).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        Ok(message)
    }

    pub async fn send_channel_message(
        &self,
        channel_id: Uuid,
        user_id: Uuid,
        content: String,
        reply_to_id: Option<Uuid>,
    ) -> Result<Message, MessageServiceError> {
        self.send_message(MessageTarget::Channel(channel_id), user_id, content, reply_to_id)
            .await
    }

    pub async fn send_conversation_message(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        content: String,
        reply_to_id: Option<Uuid>,
    ) -> Result<Message, MessageServiceError> {
        self.send_message(
            MessageTarget::Conversation(conversation_id),
            user_id,
            content,
            reply_to_id,
        )
        .await
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

        let outbox = self.outbox_event(
            message.id(),
            MessagingMessageUpdatedEvent::from_message(&message),
        )?;
        let message_for_tx = message.clone();
        let outbox_for_tx = outbox.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.messages().update(&message_for_tx).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        Ok(message)
    }

    pub async fn delete_message(
        &self,
        message_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        let mut message = self.get_message(message_id).await?;
        message.soft_delete(requester_id)?;

        let outbox = self.outbox_event(
            message.id(),
            MessagingMessageDeletedEvent::from_message(&message),
        )?;
        let message_for_tx = message.clone();
        let outbox_for_tx = outbox.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.messages().update(&message_for_tx).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_channel_messages(
        &self,
        channel_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        self.message_repo
            .delete_all_by_channel(channel_id)
            .await
            .map_err(|e| MessageServiceError::RepositoryError(e.to_string()))
    }
}
