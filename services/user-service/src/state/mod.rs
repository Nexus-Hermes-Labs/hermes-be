pub mod shared_state;
pub mod user_state;

pub use shared_state::SharedState;
pub use user_state::UserState;

use crate::application::services::{UserPrivacyService, UserProfileService};
use crate::infrastructure::persistence::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository,
};

/// Application state shared across HTTP handlers and gRPC services
#[derive(Clone)]
pub struct AppState {
    pub user: UserState,
    pub shared: SharedState,
}
