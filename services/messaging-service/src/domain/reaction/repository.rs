use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use super::entity::Reaction;

#[async_trait]
pub trait ReactionRepository: Repository<Reaction, Uuid, Error = RepositoryError> + Send + Sync {
    async fn find_by_message(&self, message_id: Uuid) -> Result<Vec<Reaction>, Self::Error>;

    async fn find_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<Option<Reaction>, Self::Error>;

    async fn count_by_message_and_emoji(
        &self,
        message_id: Uuid,
        emoji: &str,
    ) -> Result<i64, Self::Error>;

    async fn delete_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<(), Self::Error>;
}
