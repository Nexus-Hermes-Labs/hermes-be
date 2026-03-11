use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::unit_of_work::MessagingUnitOfWorkFactory;
use crate::domain::conversation::{
    validate_members, Conversation, ConversationMember, ConversationRepository,
};
use crate::domain::ConversationType;
use crate::infrastructure::NatsPublisher;

use super::error::ConversationServiceError;

pub struct ConversationService {
    conversation_repo: Arc<dyn ConversationRepository>,
    uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
    nats: Arc<NatsPublisher>,
}

impl std::fmt::Debug for ConversationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationService")
            .finish_non_exhaustive()
    }
}

impl ConversationService {
    pub fn new(
        conversation_repo: Arc<dyn ConversationRepository>,
        uow_factory: Arc<dyn MessagingUnitOfWorkFactory>,
        nats: Arc<NatsPublisher>,
    ) -> Self {
        Self {
            conversation_repo,
            uow_factory,
            nats,
        }
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

    /// Open or return the existing DM between two users (idempotent).
    pub async fn open_dm(
        &self,
        user_id_a: Uuid,
        user_id_b: Uuid,
    ) -> Result<Conversation, ConversationServiceError> {
        // Return existing DM if one exists
        if let Some(existing) = self
            .conversation_repo
            .find_dm_between(user_id_a, user_id_b)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?
        {
            return Ok(existing);
        }

        let member_ids = [user_id_a, user_id_b];
        validate_members(ConversationType::Dm, &member_ids)?;

        let conversation = Conversation::new_dm();
        let members: Vec<ConversationMember> = member_ids
            .iter()
            .map(|&uid| ConversationMember::new(conversation.id(), uid))
            .collect();

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.conversations()
            .save(&conversation)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.members()
            .save_batch(&members)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        self.nats
            .publish_conversation_created(&conversation, &member_ids)
            .await;
        Ok(conversation)
    }

    /// Create a new group DM.
    pub async fn create_group_dm(
        &self,
        member_ids: Vec<Uuid>,
    ) -> Result<Conversation, ConversationServiceError> {
        validate_members(ConversationType::GroupDm, &member_ids)?;

        let conversation = Conversation::new_group_dm();
        let members: Vec<ConversationMember> = member_ids
            .iter()
            .map(|&uid| ConversationMember::new(conversation.id(), uid))
            .collect();

        let uow = self
            .uow_factory
            .begin()
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.conversations()
            .save(&conversation)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.members()
            .save_batch(&members)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        uow.commit()
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        self.nats
            .publish_conversation_created(&conversation, &member_ids)
            .await;
        Ok(conversation)
    }

    /// Add a user to a group DM.
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
        self.conversation_repo
            .save_members(&[member])
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;

        self.nats
            .publish_conversation_member_joined(conversation_id, user_id)
            .await;
        Ok(())
    }

    /// Remove a user from a group DM (leave).
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
