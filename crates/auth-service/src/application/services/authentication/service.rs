use crate::api::dto::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::application::services::authentication::error::AuthApplicationError;
use crate::domain::user::service::PasswordService;
use crate::domain::user::{AuthUserRepository, User};
use common::jwt::JwtManager;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// AuthService
///
/// Handles user registration and authentication
pub struct AuthService<AR, PS>
where
    AR: AuthUserRepository,
    PS: PasswordService,
{
    user_repository: Arc<AR>,
    password_service: Arc<PS>,
    jwt_manager: JwtManager,
}

impl<AR: AuthUserRepository, PS: PasswordService> AuthService<AR, PS> {
    pub fn new(
        user_repository: Arc<AR>,
        password_service: Arc<PS>,
        jwt_manager: JwtManager,
    ) -> Self {
        Self {
            user_repository,
            password_service,
            jwt_manager,
        }
    }

    // ============================================
    // REGISTRATION
    // ============================================

    pub async fn register(
        &self,
        request: RegisterRequest,
    ) -> Result<AuthResponse, AuthApplicationError> {
        if !self.is_email_available(&request.email).await? {
            return Err(AuthApplicationError::EmailAlreadyExists(
                request.email.clone(),
            ));
        }

        let password_hash = self
            .password_service
            .hash_password(&request.password)
            .map_err(|e| {
                error!(error = %e, "Password hashing failed");
                AuthApplicationError::PasswordHashingFailed
            })?;

        let user = User::new(
            request.username,
            request.email,
            request.display_name,
            password_hash,
        );

        self.save_user(&user).await?;

        info!(
            user_id = %user.id(),
            email = %user.email(),
            "User registered successfully"
        );

        self.build_auth_response(&user)
    }

    // ============================================
    // LOGIN
    // ============================================

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthApplicationError> {
        let user = self
            .get_user_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                warn!(email = %request.email, "Login attempt with non-existent email");
                AuthApplicationError::InvalidCredentials
            })?;

        user.ensure_active().map_err(|_| {
            warn!(user_id = %user.id(), "Login attempt on deactivated account");
            AuthApplicationError::AccountDeactivated
        })?;

        let is_valid = self
            .password_service
            .verify_password(&request.password, user.password_hash())
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Password verification failed");
                AuthApplicationError::Internal("Password verification failed".to_string())
            })?;

        if !is_valid {
            warn!(user_id = %user.id(), "Login attempt with invalid password");
            return Err(AuthApplicationError::InvalidCredentials);
        }

        info!(
            user_id = %user.id(),
            email = %user.email(),
            "User logged in successfully"
        );

        self.build_auth_response(&user)
    }

    /// Logout user by revoking refresh token
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthApplicationError> {
        let claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token provided during logout");
                AuthApplicationError::InvalidToken
            })?;

        info!(
            user_id = %claims.sub,
            token_jti = %claims.jti,
            token_exp = claims.exp,
            "User logged out successfully"
        );

        // TODO: token blacklist

        Ok(())
    }

    /// Logout from all devices (future feature)
    #[allow(dead_code)]
    pub async fn logout_all_devices(&self, user_id: Uuid) -> Result<(), AuthApplicationError> {
        info!(user_id = %user_id, "Logout from all devices requested");
        todo!("Logout from all devices not yet implemented")
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<AuthResponse, AuthApplicationError> {
        let claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token during token refresh");
                AuthApplicationError::InvalidToken
            })?;

        let user = self.get_user_by_id(claims.sub).await?;

        user.ensure_active().map_err(|_| {
            warn!(user_id = %user.id(), "Token refresh attempt on inactive account");
            AuthApplicationError::AccountDeactivated
        })?;

        info!(user_id = %user.id(), "Token refreshed successfully");

        // TODO: old token blacklist

        self.build_auth_response(&user)
    }

    // ============================================
    // USER OPERATIONS
    // ============================================

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AuthApplicationError> {
        self.user_repository
            .find_by_email(email)
            .await
            .map(|entity| entity.map(Into::into))
            .map_err(|e| {
                error!(error = ?e, email = %email, "Database error while fetching user by email");
                AuthApplicationError::Internal(format!("Database error: {}", e))
            })
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<User, AuthApplicationError> {
        self.user_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                error!(error = ?e, user_id = %id, "Database error while fetching user by id");
                AuthApplicationError::Internal(format!("Database error: {}", e))
            })?
            .map(Into::into)
            .ok_or_else(|| {
                warn!(user_id = %id, "User not found");
                AuthApplicationError::UserNotFound(id)
            })
    }

    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, AuthApplicationError> {
        self.user_repository
            .find_by_username(username)
            .await
            .map(|entity| entity.map(Into::into))
            .map_err(|e| {
                error!(error = ?e, username = %username, "Database error while fetching user by username");
                AuthApplicationError::Internal(format!("Database error: {}", e))
            })
    }

    async fn save_user(&self, user: &User) -> Result<(), AuthApplicationError> {
        self.user_repository.save(&user).await.map_err(|e| {
            error!(
                error = ?e,
                user_id = %user.id(),
                email = %user.email(),
                "Database error while saving user"
            );
            AuthApplicationError::Internal(format!("Failed to save user: {}", e))
        })
    }

    // ============================================
    // HELPER METHODS
    // ============================================

    fn build_auth_response(&self, user: &User) -> Result<AuthResponse, AuthApplicationError> {
        let role_str = user.role().as_str().to_string();

        let access_token = self
            .jwt_manager
            .create_user_token(user.id(), user.email().to_string(), role_str.clone(), 6)
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Failed to create access token");
                AuthApplicationError::TokenGenerationFailed(e)
            })?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(user.id(), user.email().to_string(), role_str.clone(), 30)
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Failed to create refresh token");
                AuthApplicationError::TokenGenerationFailed(e)
            })?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            expires_in: 6 * 60 * 60,
            user: UserResponse {
                id: user.id().to_string(),
                email: user.email().to_string(),
                display_name: user.display_name().to_string(),
                username: user.username().to_string(),
                role: role_str,
                is_active: user.is_active(),
                email_verified: user.is_email_verified(),
            },
        })
    }

    async fn is_email_available(&self, email: &str) -> Result<bool, AuthApplicationError> {
        Ok(self.get_user_by_email(email).await?.is_none())
    }
}
