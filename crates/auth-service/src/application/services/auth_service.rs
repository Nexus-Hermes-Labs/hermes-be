use crate::api::dto::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::domain::user::{AuthUserRepository, User};
use crate::infrastructure::persistence::user::UserMapper;
use anyhow::Context;
use common::utils::{hash_password, verify_password};
use common::AppError;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use common::jwt::JwtManager;

/// AuthService
///
/// Handles user registration and authentication
pub struct AuthService<R: AuthUserRepository> {
    user_repository: Arc<R>,
    jwt_manager: JwtManager,
}

impl<R: AuthUserRepository> AuthService<R> {
    pub fn new(user_repository: Arc<R>, jwt_manager: JwtManager) -> Self {
        Self {
            user_repository,
            jwt_manager,
        }
    }

    // ============================================
    // REGISTRATION
    // ============================================

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        // Check if user exists
        if !self.is_email_available(&request.email).await? {
            return Err(AppError::Conflict("Email already in use".to_string()));
        }

        if !self.is_username_available(&request.username).await? {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }

        // Hash password
        let password_hash = hash_password(&request.password).map_err(|e| {
            AppError::InternalServerError(
                anyhow::anyhow!("Password hashing failed: {}", e).to_string(),
            )
        })?;

        // Create user
        let user = User::new(request.email, request.username, password_hash);

        // Save user
        self.save_user(&user).await?;

        // Generate tokens
        self.build_auth_response(&user).await
    }

    // ============================================
    // LOGIN
    // ============================================

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        // Find user
        let user = self
            .get_user_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let password_valid =
            verify_password(&request.password, &user.password_hash).map_err(|e| {
                AppError::InternalServerError(format!("Password verification failed: {}", e))
            })?;

        if !password_valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // Check user status
        if !user.is_active {
            return Err(AppError::Forbidden("Account is deactivated".to_string()));
        }

        // Generate tokens
        self.build_auth_response(&user).await
    }

    /// Logout user by revoking refresh token
    ///
    /// Current implementation:
    /// 1. Validates refresh token
    /// 2. Logs audit event
    /// 3. Returns success to client
    ///
    /// Future enhancements (when needed):
    /// - Add token to blacklist database
    /// - Invalidate token family (for rotation)
    /// - Trigger logout webhooks/events
    /// - Update user's last_logout_at timestamp
    ///
    /// # Arguments
    /// * `refresh_token` - The refresh token to revoke
    ///
    /// # Returns
    /// * `Ok(())` - Token successfully validated and logged
    /// * `Err(AppError)` - Invalid or expired token
    pub async fn logout(&self, refresh_token: String) -> Result<(), AppError> {
        // 1. Verify and decode refresh token
        let claims = self
            .jwt_manager
            .verify_refresh_token(&refresh_token)
            .map_err(|e| {
                warn!(
                    error = %e,
                    "Invalid refresh token provided during logout"
                );
                AppError::Unauthorized("Invalid or expired refresh token".to_string())
            })?;

        // 2. Audit logging (important for security)
        info!(
            user_id = %claims.sub,
            token_jti = %claims.jti,
            token_exp = claims.exp,
            "User logged out successfully"
        );

        // TODO:
        // 3. Future: Blacklist token
        // When blacklist is implemented, uncomment:
        //
        // let expires_at = DateTime::from_timestamp(claims.exp, 0)
        //     .unwrap_or_else(|| Utc::now() + Duration::days(30));
        //
        // self.blacklist_repository
        //     .blacklist_token(
        //         claims.sub,
        //         claims.jti,
        //         "refresh",
        //         expires_at,
        //     )
        //     .await
        //     .map_err(|e| {
        //         error!("Failed to blacklist token: {}", e);
        //         AppError::Internal(e.into())
        //     })?;

        // 4. Future: Trigger events
        // self.event_bus.publish(UserLoggedOutEvent {
        //     user_id: claims.sub,
        //     logged_out_at: Utc::now(),
        // }).await?;

        Ok(())
    }

    /// Logout from all devices (future feature)
    ///
    /// This would revoke all refresh tokens for a user
    /// Requires token family tracking or user-level revocation
    #[allow(dead_code)]
    pub async fn logout_all_devices(&self, user_id: Uuid) -> Result<(), AppError> {
        info!(user_id = %user_id, "Logout from all devices requested");

        // TODO: Future implementation:
        // 1. Add user_id to global blacklist with timestamp
        // 2. All tokens issued before this timestamp are invalid
        // 3. Or: Track and revoke all refresh tokens for user

        todo!("Logout from all devices not yet implemented")
    }
    
        pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        // Verify refresh token
        let claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        // Get user
        let user_id = claims.sub;
        let user = self.get_user_by_id(user_id).await?;

        // Check user status
        if !user.is_active {
            return Err(AppError::Unauthorized("Account is not active".to_string()));
        }

        // TODO: Add token to blacklist here

        // Generate new tokens
        self.build_auth_response(&user).await
    }

    // ============================================
    // USER OPERATIONS
    // ============================================
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let entity = self
            .user_repository
            .find_by_email(email)
            .await
            .map_err(|_| {
                AppError::InternalServerError("Faield to fetch user from database".to_string())
            })?;
        Ok(entity.map(UserMapper::to_domain))
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<User, AppError> {
        let entity = self
            .user_repository
            .find_by_id(id)
            .await
            .context("Failed to fetch user")
            .map_err(|_| {
                AppError::InternalServerError(
                    "Failed to fetch user from database: {:?}".to_string(),
                )
            })?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "User".to_string(),
            })?;

        Ok(UserMapper::to_domain(entity))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let entity = self
            .user_repository
            .find_by_username(username)
            .await
            .map_err(|_| {
                AppError::InternalServerError("Failed to fetch user from database".to_string())
            })?;
        Ok(entity.map(UserMapper::to_domain))
    }

    async fn save_user(&self, user: &User) -> Result<(), AppError> {
        let entity = UserMapper::to_entity(user);
        self.user_repository.save(&entity).await.map_err(|_| {
            AppError::InternalServerError("Failed to save user to database".to_string())
        })?;
        Ok(())
    }

    // ============================================
    // HELPER METHODS (for AuthService)
    // ============================================
    async fn build_auth_response(&self, user: &User) -> Result<AuthResponse, AppError> {
        let role_str = format!("{:?}", user.role).to_lowercase();

        let access_token = self
            .jwt_manager
            .create_user_token(user.id, user.email.clone(), role_str.clone(), 6)
            .map_err(|e| {
                AppError::InternalServerError(
                    anyhow::anyhow!("Token generation failed: {}", e).to_string(),
                )
            })?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(user.id, user.email.clone(), role_str.clone(), 30)
            .map_err(|e| {
                AppError::InternalServerError(
                    anyhow::anyhow!("Refresh token generation failed: {}", e).to_string(),
                )
            })?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: UserResponse {
                id: user.id.to_string(),
                email: user.email.clone(),
                username: user.username.clone(),
                role: role_str,
                is_active: user.is_active,
                email_verified: user.email_verified,
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
