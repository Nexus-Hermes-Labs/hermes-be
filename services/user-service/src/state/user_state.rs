use crate::application::{UserPrivacyService, UserProfileService, UserRelationshipService};
use std::sync::Arc;

/// User domain state
///
/// Contains pre-composed user service and its dependencies.
#[derive(Clone)]
pub struct UserState {
    // Services
    pub user_profile_service: Arc<UserProfileService>,
    pub user_privacy_service: Arc<UserPrivacyService>,
    pub relationship_service: Arc<UserRelationshipService>,
}

impl UserState {
    pub fn new(
        user_profile_service: Arc<UserProfileService>,
        user_privacy_service: Arc<UserPrivacyService>,
        relationship_service: Arc<UserRelationshipService>,
    ) -> Self {
        Self {
            user_profile_service,
            user_privacy_service,
            relationship_service,
        }
    }
}
