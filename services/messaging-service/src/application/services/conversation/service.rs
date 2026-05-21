use std::sync::Arc;
use uuid::Uuid;

use common::domain::event::IntoEventEnvelope;
use common::infrastructure::outbox::{AggregateType, NewOutboxEvent, OutboxEventType};

use crate::application::events::{
    MessagingConversationCreatedEvent, MessagingConversationMemberJoinedEvent,
};
use crate::application::ports::unit_of_work::MessagingUnitOfWorkFactory;
use crate::domain::conversation::{
    validate_members, Conversation, ConversationMember, ConversationRepository,
};
use crate::domain::ConversationType;

use super::error::ConversationServiceError;

pub struct ConversationService {
    service_name: String,
    conversation_repo: Arc<dyn ConversationRepository>,
    uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
}

impl std::fmt::Debug for ConversationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationService")
            .finish_non_exhaustive()
    }
}

impl ConversationService {
    pub fn new(
        service_name: impl Into<String>,
        conversation_repo: Arc<dyn ConversationRepository>,
        uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            conversation_repo,
            uow_factory,
        }
    }

    fn outbox_event(
        &self,
        aggregate_id: Uuid,
        event: impl IntoEventEnvelope,
        event_type: OutboxEventType,
    ) -> Result<NewOutboxEvent, ConversationServiceError> {
        let envelope = event.into_envelope(&self.service_name);
        let payload = serde_json::to_value(&envelope)
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        Ok(NewOutboxEvent {
            id: envelope.event_id,
            aggregate_id,
            aggregate_type: AggregateType::MessagingConversation,
            event_type,
            payload,
        })
    }

    // ── Queries ───────────────────────────────────────────────────────────

    pub async fn get_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<Conversation, ConversationServiceError> {
        self.conversation_repo
            .find_by_id(conversation_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?
            .ok_or(ConversationServiceError::NotFound)
    }

    pub async fn get_user_conversations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Conversation>, ConversationServiceError> {
        self.conversation_repo
            .find_by_member(user_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))
    }

    pub async fn get_conversation_members(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<ConversationMember>, ConversationServiceError> {
        self.conversation_repo
            .find_members(conversation_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))
    }

    // ── Commands ──────────────────────────────────────────────────────────

    async fn create_conversation(
        &self,
        conversation: Conversation,
        member_ids: Vec<Uuid>,
    ) -> Result<Conversation, ConversationServiceError> {
        let members: Vec<ConversationMember> = member_ids
            .iter()
            .map(|&uid| ConversationMember::new(conversation.id(), uid))
            .collect();

        let outbox = self.outbox_event(
            conversation.id(),
            MessagingConversationCreatedEvent::new(&conversation, &member_ids),
            OutboxEventType::MessagingConversationCreated,
        )?;
        let conv_for_tx = conversation.clone();
        let members_for_tx = members.clone();
        let outbox_for_tx = outbox.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.conversations().save(&conv_for_tx).await?;
                    uow.members().save_batch(&members_for_tx).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        Ok(conversation)
    }

    pub async fn open_dm(
        &self,
        user_id_a: Uuid,
        user_id_b: Uuid,
    ) -> Result<Conversation, ConversationServiceError> {
        if let Some(existing) = self
            .conversation_repo
            .find_dm_between(user_id_a, user_id_b)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?
        {
            return Ok(existing);
        }

        let member_ids = vec![user_id_a, user_id_b];
        validate_members(ConversationType::Dm, &member_ids)?;
        self.create_conversation(Conversation::new_dm(), member_ids)
            .await
    }

    pub async fn create_group_dm(
        &self,
        member_ids: Vec<Uuid>,
    ) -> Result<Conversation, ConversationServiceError> {
        validate_members(ConversationType::GroupDm, &member_ids)?;
        self.create_conversation(Conversation::new_group_dm(), member_ids)
            .await
    }

    pub async fn add_member(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ConversationServiceError> {
        let conversation = self.get_conversation(conversation_id).await?;

        if conversation.is_dm() {
            return Err(ConversationServiceError::DomainError(
                crate::domain::ConversationError::InvalidConversationType,
            ));
        }

        let already_member = self
            .conversation_repo
            .is_member(conversation_id, user_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        if already_member {
            return Err(ConversationServiceError::AlreadyMember);
        }

        let member = ConversationMember::new(conversation_id, user_id);
        let outbox = self.outbox_event(
            conversation_id,
            MessagingConversationMemberJoinedEvent {
                conversation_id,
                user_id,
            },
            OutboxEventType::MessagingConversationMemberJoined,
        )?;
        let outbox_for_tx = outbox.clone();
        let member_for_tx = member.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.members().save_batch(&[member_for_tx]).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_member(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ConversationServiceError> {
        let is_member = self
            .conversation_repo
            .is_member(conversation_id, user_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        if !is_member {
            return Err(ConversationServiceError::NotMember);
        }

        self.conversation_repo
            .remove_member(conversation_id, user_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))
    }
}
