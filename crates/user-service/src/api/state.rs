use crate::application::services::user::service::UserApplicationService;
use crate::infrastructure::persistence::postgres::user_repository::repository::PostgresUserRepository;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_service: Arc<UserApplicationService<PostgresUserRepository>>,
}

impl AppState {
    pub fn new(db: PgPool, user_service: UserApplicationService<PostgresUserRepository>) -> Self {
        Self {
            db,
            user_service: Arc::new(user_service),
        }
    }
}
