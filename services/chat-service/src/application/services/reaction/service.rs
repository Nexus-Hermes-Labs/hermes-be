use std::sync::Arc;
use uuid::Uuid;

use common::domain::event::IntoEventEnvelope;
use common::infrastructure::outbox::NewOutboxEvent;

use crate::application::events::{
    ChatReactionAddedEvent, ChatReactionRemovedEvent, AGGREGATE_TYPE_REACTION,
};
use crate::application::ports::ChatUnitOfWorkFactory;
use crate::domain::message::MessageRepository;
use crate::domain::reaction::{Reaction, ReactionRepository};
use crate::domain::Emoji;

use super::error::ReactionServiceError;

pub struct ReactionService {
    service_name: String,
    reaction_repo: Arc<dyn ReactionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    uow_factory: Arc<dyn ChatUnitOfWorkFactory>,
}

impl std::fmt::Debug for ReactionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactionService").finish_non_exhaustive()
    }
}

impl ReactionService {
    pub fn new(
        service_name: impl Into<String>,
        reaction_repo: Arc<dyn ReactionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        uow_factory: Arc<dyn ChatUnitOfWorkFactory>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            reaction_repo,
            message_repo,
            uow_factory,
        }
    }

    fn outbox_event(
        &self,
        aggregate_id: Uuid,
        event: impl IntoEventEnvelope,
    ) -> Result<NewOutboxEvent, ReactionServiceError> {
        let event_type = event.event_type().to_string();
        let envelope = event.into_envelope(&self.service_name);
        let payload = serde_json::to_value(&envelope)
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;
        Ok(NewOutboxEvent {
            id: envelope.event_id,
            aggregate_id,
            aggregate_type: AGGREGATE_TYPE_REACTION.to_string(),
            event_type,
            payload,
        })
    }

    pub async fn add_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji_str: String,
    ) -> Result<Reaction, ReactionServiceError> {
        self.message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?
            .ok_or(ReactionServiceError::MessageNotFound)?;

        let emoji = Emoji::new(emoji_str).map_err(ReactionServiceError::DomainError)?;

        let existing = self
            .reaction_repo
            .find_by_message_user_emoji(message_id, user_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        if existing.is_some() {
            return Err(ReactionServiceError::AlreadyReacted);
        }

        let reaction = Reaction::new(message_id, user_id, emoji);
        let outbox = self.outbox_event(
            reaction.message_id(),
            ChatReactionAddedEvent::from_reaction(&reaction),
        )?;
        let reaction_for_tx = reaction.clone();
        let outbox_for_tx = outbox.clone();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.reactions().save(&reaction_for_tx).await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        Ok(reaction)
    }

    pub async fn remove_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji_str: &str,
    ) -> Result<(), ReactionServiceError> {
        let existing = self
            .reaction_repo
            .find_by_message_user_emoji(message_id, user_id, emoji_str)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        if existing.is_none() {
            return Err(ReactionServiceError::ReactionNotFound);
        }

        let outbox = self.outbox_event(
            message_id,
            ChatReactionRemovedEvent {
                message_id,
                user_id,
                emoji: emoji_str.to_string(),
            },
        )?;
        let outbox_for_tx = outbox.clone();
        let emoji_for_tx = emoji_str.to_string();

        self.uow_factory
            .transaction(Box::new(move |uow| {
                Box::pin(async move {
                    uow.reactions()
                        .delete_by_message_user_emoji(message_id, user_id, &emoji_for_tx)
                        .await?;
                    uow.outbox().save(&outbox_for_tx).await?;
                    Ok(())
                })
            }))
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_reactions(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<Reaction>, ReactionServiceError> {
        self.reaction_repo
            .find_by_message(message_id)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))
    }
}
