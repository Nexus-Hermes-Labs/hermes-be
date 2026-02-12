use async_trait::async_trait;
use uuid::Uuid;
use common::infrastructure::persistence::repository::Repository;

use super::entity::UserPrivacySettings;

/// User privacy settings repository
#[async_trait]
pub trait UserPrivacyRepository: Repository<UserPrivacySettings, Uuid> + Send + Sync {
    // Inherits from Repository<UserPrivacySettings, Uuid>:
    // - find_by_id(id: Uuid) -> Result<Option<UserPrivacySettings>>
    // - save(entity: &UserPrivacySettings) -> Result<()>
    // - update(entity: &UserPrivacySettings) -> Result<()>
    // - delete(id: Uuid) -> Result<()>
    // - exists(id: Uuid) -> Result<bool>
    // - count() -> Result<i64>
    
    // No domain-specific methods needed for MVP
    // Privacy settings are only accessed by user_id (which is the PK)
}
