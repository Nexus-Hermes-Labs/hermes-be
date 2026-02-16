use crate::application::{UserPrivacyService, UserProfileService, UserRelationshipService};
use crate::infrastructure::persistence::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository, PostgresUserRelationshipRepository,
};
use std::sync::Arc;

/// User domain state
///
/// Contains pre-composed authentication service and its dependencies.
/// Generic over trait implementations to maintain dependency inversion.
#[derive(Clone)]
pub struct UserState {
    // Services
    pub user_profile_service: Arc<UserProfileService<PostgresUserProfileRepository>>,
    pub user_privacy_service: Arc<UserPrivacyService<PostgresUserPrivacyRepository>>,
    pub relationship_service: Arc<UserRelationshipService<PostgresUserRelationshipRepository>>,
}

impl UserState {
    pub fn new(
        user_profile_service: Arc<UserProfileService<PostgresUserProfileRepository>>,
        user_privacy_service: Arc<UserPrivacyService<PostgresUserPrivacyRepository>>,
        relationship_service: Arc<UserRelationshipService<PostgresUserRelationshipRepository>>,
    ) -> Self {
        Self {
            user_profile_service,
            user_privacy_service,
            relationship_service,
        }
    }
}
