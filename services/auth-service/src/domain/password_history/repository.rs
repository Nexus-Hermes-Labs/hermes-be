use super::PasswordHistory;
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use uuid::Uuid;

#[async_trait]
pub trait PasswordHistoryRepository: Send + Sync {
    async fn save(&self, entry: &PasswordHistory) -> Result<(), RepositoryError>;

    async fn find_recent_by_credential(
        &self,
        credential_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PasswordHistory>, RepositoryError>;

    async fn delete_oldest_beyond_limit(
        &self,
        credential_id: Uuid,
        keep_count: i64,
    ) -> Result<u64, RepositoryError>;
}
