use common::jwt::JwtManager;
use sqlx::PgPool;
use std::sync::Arc;
use crate::application::services::auth::service::AuthService;
use crate::infrastructure::persistence::postgres::user_repository::PostgresAuthUserRepository;
use crate::infrastructure::security::argon2_password_service::Argon2PasswordService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth_service: Arc<AuthService<PostgresAuthUserRepository, Argon2PasswordService>>,
    pub jwt_manager: Arc<JwtManager>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        auth_service: AuthService<PostgresAuthUserRepository, Argon2PasswordService>,
        jwt_manager: JwtManager,
    ) -> Self {
        Self {
            db,
            auth_service: Arc::new(auth_service),
            jwt_manager: Arc::new(jwt_manager),
        }
    }
}
