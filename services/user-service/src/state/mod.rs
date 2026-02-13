mod shared_state;
mod user_state;

use std::sync::Arc;

use sqlx::PgPool;

use crate::application::services::{UserPrivacyService, UserProfileService};
use crate::infrastructure::persistence::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository,
};
use crate::state::shared_state::SharedState;
use crate::state::user_state::UserState;

/// Application state shared across HTTP handlers and gRPC services
#[derive(Clone)]
pub struct AppState {
    pub user: UserState,
    pub shared: SharedState,
}
