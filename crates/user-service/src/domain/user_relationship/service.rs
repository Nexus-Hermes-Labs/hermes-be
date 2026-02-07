use super::entity::UserRelationship;
use crate::domain::user::entity::User;
use crate::domain::user::valueobject::UserSnapshot;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user_relationship::valueobject::UserRelationshipWithTarget;
use std::collections::HashMap;
use uuid::Uuid;

/// UserRelationship Domain Service
/// Handles cross-aggregate operations
pub struct UserRelationshipDomainService;

impl UserRelationshipDomainService {
    /// Enrich relationships with target user details
    pub fn enrich_with_targets(
        relationships: Vec<UserRelationship>,
        users: Vec<User>,
    ) -> Vec<UserRelationshipWithTarget> {
        let user_map: HashMap<Uuid, User> = users.into_iter().map(|user| (user.id, user)).collect();

        relationships
            .into_iter()
            .filter_map(|rel| {
                user_map
                    .get(&rel.target_user_id())
                    .map(|user| UserRelationshipWithTarget::new(rel, UserSnapshot::from_user(user)))
            })
            .collect()
    }

    /// Extract user IDs from relationships
    pub fn extract_target_ids(relationships: &[UserRelationship]) -> Vec<Uuid> {
        relationships
            .iter()
            .map(|rel| rel.target_user_id().clone())
            .collect()
    }
}
