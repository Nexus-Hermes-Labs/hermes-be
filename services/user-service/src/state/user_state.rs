use crate::application::{UserPrivacyService, UserProfileService};
use crate::infrastructure::persistence::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository,
};
use common::infrastructure::messaging::EventPublisher;
use common::infrastructure::security::jwt_manager::JwtManager;
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
}

impl UserState {
    pub fn new(
        user_profile_service: Arc<UserProfileService<PostgresUserProfileRepository>>,
        user_privacy_service: Arc<UserPrivacyService<PostgresUserPrivacyRepository>>,
    ) -> Self {
        Self {
            user_profile_service,
            user_privacy_service,
        }
    }
}
