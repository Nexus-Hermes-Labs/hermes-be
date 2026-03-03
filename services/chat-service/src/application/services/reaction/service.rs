use std::sync::Arc;
use uuid::Uuid;

use crate::domain::message::MessageRepository;
use crate::domain::reaction::{Reaction, ReactionRepository};
use crate::domain::Emoji;
use crate::infrastructure::events::NatsPublisher;

use super::error::ReactionServiceError;

/// Reaction application service.
pub struct ReactionService {
    reaction_repo: Arc<dyn ReactionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    nats: Arc<NatsPublisher>,
}

impl std::fmt::Debug for ReactionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactionService").finish_non_exhaustive()
    }
}

impl ReactionService {
    pub fn new(
        reaction_repo: Arc<dyn ReactionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        nats: Arc<NatsPublisher>,
    ) -> Self {
        Self {
            reaction_repo,
            message_repo,
            nats,
        }
    }

    // ── Add reaction ──────────────────────────────────────────────────────

    pub async fn add_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji_str: String,
    ) -> Result<Reaction, ReactionServiceError> {
        // Verify the message exists
        self.message_repo
            .find_by_id(message_id)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?
            .ok_or(ReactionServiceError::MessageNotFound)?;

        let emoji = Emoji::new(emoji_str).map_err(ReactionServiceError::DomainError)?;

        // Deduplication check
        let existing = self
            .reaction_repo
            .find_by_message_user_emoji(message_id, user_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        if existing.is_some() {
            return Err(ReactionServiceError::AlreadyReacted);
        }

        let reaction = Reaction::new(message_id, user_id, emoji);

        self.reaction_repo
            .save(&reaction)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        self.nats.publish_reaction_added(&reaction).await;

        Ok(reaction)
    }

    // ── Remove reaction ───────────────────────────────────────────────────

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

        self.reaction_repo
            .delete_by_message_user_emoji(message_id, user_id, emoji_str)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;

        self.nats
            .publish_reaction_removed(message_id, user_id, emoji_str)
            .await;

        Ok(())
    }

    // ── Get reactions ─────────────────────────────────────────────────────

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
