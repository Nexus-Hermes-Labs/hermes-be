use std::sync::Arc;
use uuid::Uuid;

use crate::domain::reaction::{Reaction, ReactionRepository};
use crate::domain::Emoji;
use crate::infrastructure::NatsPublisher;

use super::error::ReactionServiceError;

pub struct ReactionService {
    reaction_repo: Arc<dyn ReactionRepository>,
    nats: Arc<NatsPublisher>,
}

impl std::fmt::Debug for ReactionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactionService").finish_non_exhaustive()
    }
}

impl ReactionService {
    pub fn new(reaction_repo: Arc<dyn ReactionRepository>, nats: Arc<NatsPublisher>) -> Self {
        Self {
            reaction_repo,
            nats,
        }
    }

    // ── Queries ───────────────────────────────────────────────────────────

    pub async fn get_message_reactions(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<Reaction>, ReactionServiceError> {
        self.reaction_repo
            .find_by_message(message_id)
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))
    }

    pub async fn count_emoji_reactions(
        &self,
        message_id: Uuid,
        emoji_str: String,
    ) -> Result<i64, ReactionServiceError> {
        let emoji = Emoji::new(emoji_str)?;
        self.reaction_repo
            .count_by_message_and_emoji(message_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))
    }

    // ── Commands ──────────────────────────────────────────────────────────

    pub async fn add_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji_str: String,
    ) -> Result<Reaction, ReactionServiceError> {
        let emoji = Emoji::new(emoji_str)?;

        if self
            .reaction_repo
            .find_by_message_user_emoji(message_id, user_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?
            .is_some()
        {
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

    pub async fn remove_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji_str: String,
    ) -> Result<(), ReactionServiceError> {
        let emoji = Emoji::new(emoji_str)?;

        // Verify it exists before deletion
        self.reaction_repo
            .find_by_message_user_emoji(message_id, user_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?
            .ok_or(ReactionServiceError::NotFound)?;

        self.reaction_repo
            .delete_by_message_user_emoji(message_id, user_id, emoji.as_str())
            .await
            .map_err(|e| ReactionServiceError::RepositoryError(e.to_string()))?;
        self.nats
            .publish_reaction_removed(message_id, user_id, emoji.as_str())
            .await;
        Ok(())
    }
}
