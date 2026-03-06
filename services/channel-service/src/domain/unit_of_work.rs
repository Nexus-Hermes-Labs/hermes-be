use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::channel::Channel;

/// Transactional writer for the `channels` table.
#[async_trait]
pub trait ChannelWriter: Send + Sync {
    async fn save(&self, channel: &Channel) -> Result<(), RepositoryError>;
}

