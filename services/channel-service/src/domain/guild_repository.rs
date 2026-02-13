use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::guild::Guild;
use common::infrastructure::persistence::repository::Repository;

#[async_trait]
pub trait GuildRepository: Repository<Guild, Uuid> {
    // Add any guild-specific methods here
}
