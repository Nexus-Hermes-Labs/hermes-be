use crate::application::services::user::service::UserApplicationService;
use crate::infrastructure::persistence::postgres::user_repository::repository::PostgresUserRepository;
use sqlx::PgPool;
use std::sync::Arc;
use common::jwt::JwtManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_service: Arc<UserApplicationService<PostgresUserRepository>>,
    pub jwt_manager: Arc<JwtManager>,

}

impl axum::extract::FromRef<AppState> for Arc<JwtManager> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_manager.clone()
    }
}

impl AppState {
    pub fn new(db: PgPool, user_service: UserApplicationService<PostgresUserRepository>, jwt_manager: JwtManager) -> Self {
        Self {
            db,
            user_service: Arc::new(user_service),
            jwt_manager: Arc::new(jwt_manager),
        }
    }
}
