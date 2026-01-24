use crate::application::AuthService;
use crate::infrastructure::persistence::PostgresAuthUserRepository;
use common::jwt::JwtManager;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth_service: Arc<AuthService<PostgresAuthUserRepository>>,
    pub jwt_manager: Arc<JwtManager>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        auth_service: AuthService<PostgresAuthUserRepository>,
        jwt_manager: JwtManager,
    ) -> Self {
        Self {
            db,
            auth_service: Arc::new(auth_service),
            jwt_manager: Arc::new(jwt_manager),
        }
    }
}
