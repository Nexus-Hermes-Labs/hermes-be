use crate::domain::user::service::PasswordService;
use crate::domain::user::{AuthUserRepository, User};
use common::jwt::JwtManager;
use common::AppError;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;
use crate::api::dto::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::infrastructure::persistence::postgres::user_repository::models::UserRow;

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

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        if !self.is_email_available(&request.email).await? {
            return Err(AppError::Conflict("Email already in use".to_string()));
        }

        if !self.is_username_available(&request.username).await? {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }

        let password_hash = self
            .password_service
            .hash_password(&request.password)
            .map_err(|e| {
                error!(error = %e, "Password hashing failed");
                AppError::InternalServerError("Failed to process password".to_string())
            })?;

        let user = User::new(request.username, request.email, password_hash);

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

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self
            .get_user_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                warn!(email = %request.email, "Login attempt with non-existent email");
                AppError::Unauthorized("Invalid credentials".to_string())
            })?;

        user.ensure_active().map_err(|_| {
            warn!(user_id = %user.id(), "Login attempt on deactivated account");
            AppError::Forbidden("Account is deactivated".to_string())
        })?;

        let is_valid = self
            .password_service
            .verify_password(&request.password, user.password_hash())
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Password verification failed");
                AppError::InternalServerError("Authentication error".to_string())
            })?;

        if !is_valid {
            warn!(user_id = %user.id(), "Login attempt with invalid password");
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        info!(
            user_id = %user.id(),
            email = %user.email(),
            "User logged in successfully"
        );

        self.build_auth_response(&user)
    }

    /// Logout user by revoking refresh token
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        let claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token provided during logout");
                AppError::Unauthorized("Invalid or expired refresh token".to_string())
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
    pub async fn logout_all_devices(&self, user_id: Uuid) -> Result<(), AppError> {
        info!(user_id = %user_id, "Logout from all devices requested");
        todo!("Logout from all devices not yet implemented")
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        let claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token during token refresh");
                AppError::Unauthorized("Invalid refresh token".to_string())
            })?;

        let user = self.get_user_by_id(claims.sub).await?;

        user.ensure_active().map_err(|_| {
            warn!(user_id = %user.id(), "Token refresh attempt on inactive account");
            AppError::Unauthorized("Account is not active".to_string())
        })?;

        info!(user_id = %user.id(), "Token refreshed successfully");

        // TODO: old token blacklist

        self.build_auth_response(&user)
    }

    // ============================================
    // USER OPERATIONS
    // ============================================

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.user_repository
            .find_by_email(email)
            .await
            .map(|entity| entity.map(Into::into))
            .map_err(|e| {
                error!(error = ?e, email = %email, "Database error while fetching user by email");
                AppError::InternalServerError("Failed to fetch user".to_string())
            })
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<User, AppError> {
        self.user_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                error!(error = ?e, user_id = %id, "Database error while fetching user by id");
                AppError::InternalServerError("Failed to fetch user".to_string())
            })?
            .map(Into::into)
            .ok_or_else(|| {
                warn!(user_id = %id, "User not found");
                AppError::NotFound {
                    entity_type: "User".to_string(),
                }
            })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.user_repository
            .find_by_username(username)
            .await
            .map(|entity| entity.map(Into::into))
            .map_err(|e| {
                error!(error = ?e, username = %username, "Database error while fetching user by username");
                AppError::InternalServerError("Failed to fetch user".to_string())
            })
    }

    async fn save_user(&self, user: &User) -> Result<(), AppError> {
        self.user_repository.save(&user).await.map_err(|e| {
            error!(
                error = ?e,
                user_id = %user.id(),
                email = %user.email(),
                "Database error while saving user"
            );
            AppError::InternalServerError("Failed to save user".to_string())
        })
    }

    // ============================================
    // HELPER METHODS
    // ============================================

    fn build_auth_response(&self, user: &User) -> Result<AuthResponse, AppError> {
        let role_str = user.role().as_str().to_string();

        let access_token = self
            .jwt_manager
            .create_user_token(user.id(), user.email().to_string(), role_str.clone(), 6)
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Failed to create access token");
                AppError::InternalServerError("Token generation failed".to_string())
            })?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(user.id(), user.email().to_string(), role_str.clone(), 30)
            .map_err(|e| {
                error!(error = %e, user_id = %user.id(), "Failed to create refresh token");
                AppError::InternalServerError("Token generation failed".to_string())
            })?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: UserResponse {
                id: user.id().to_string(),
                email: user.email().to_string(),
                username: user.username().to_string(),
                role: role_str,
                is_active: user.is_active(),
                email_verified: user.is_email_verified(),
            },
        })
    }

    async fn is_email_available(&self, email: &str) -> Result<bool, AppError> {
        Ok(self.get_user_by_email(email).await?.is_none())
    }

    async fn is_username_available(&self, username: &str) -> Result<bool, AppError> {
        Ok(self.get_user_by_username(username).await?.is_none())
    }
}