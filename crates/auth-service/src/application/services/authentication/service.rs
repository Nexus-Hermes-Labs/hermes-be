use crate::presentation::http::dto::auth::{
    AuthResponse, LoginRequest, RefreshTokenResponse, RegisterRequest, UserResponse,
};
use crate::application::services::authentication::error::AuthApplicationError;
use crate::domain::user::service::PasswordService;
use crate::domain::user::{AuthUserRepository, User};
use common::jwt::JwtManager;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;
use crate::domain::user::valueobject::PasswordHashVO;
// ============================================
// CONSTANTS
// ============================================

/// Access token expiry in hours
const ACCESS_TOKEN_EXPIRY_HOURS: u64 = 6;

/// Refresh token expiry in days
const REFRESH_TOKEN_EXPIRY_DAYS: u64 = 30;

// ============================================
// SERVICE
// ============================================

/// Authentication Service
///
/// Handles user registration, login, logout, and token management
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
}

// ============================================
// PUBLIC API - AUTHENTICATION OPERATIONS
// ============================================

impl<AR: AuthUserRepository, PS: PasswordService> AuthService<AR, PS> {
    /// Register a new user
    pub async fn register(
        &self,
        request: RegisterRequest,
    ) -> Result<AuthResponse, AuthApplicationError> {
        // Validate email availability
        if !self.is_email_available(&request.email).await? {
            return Err(AuthApplicationError::EmailAlreadyExists(request.email));
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user entity
        let user = User::new(
            request.username,
            request.email,
            request.display_name,
            password_hash,
        );

        // Persist user
        self.save_user(&user).await?;

        info!(
            user_id = %user.id(),
            username = %user.username(),
            email = %user.email(),
            "User registered successfully"
        );

        // Generate tokens and return
        self.create_auth_response(&user)
    }

    /// Login with email and password
    pub async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<AuthResponse, AuthApplicationError> {
        // Fetch user
        let user = self
            .find_user_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                warn!(email = %request.email, "Login attempt with non-existent email");
                AuthApplicationError::InvalidCredentials
            })?;

        // Verify account is active
        self.ensure_user_active(&user)?;

        // Verify password
        self.verify_password(&request.password, user.password_hash())?;

        info!(
            user_id = %user.id(),
            username = %user.username(),
            email = %user.email(),
            "User logged in successfully"
        );

        // Generate tokens and return
        self.create_auth_response(&user)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResponse, AuthApplicationError> {
        // Verify refresh token
        let claims = self.verify_refresh_token(refresh_token)?;

        // Fetch user
        let user = self.find_user_by_id(claims.sub).await?;

        // Verify account is still active
        self.ensure_user_active(&user)?;

        info!(
            user_id = %user.id(),
            username = %user.username(),
            "Token refreshed successfully"
        );

        // TODO: Blacklist old refresh token

        // Generate new tokens
        self.create_refresh_response(&user)
    }

    /// Logout user by invalidating refresh token
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthApplicationError> {
        // Verify token is valid
        let claims = self.verify_refresh_token(refresh_token)?;

        info!(
            user_id = %claims.sub,
            token_jti = %claims.jti,
            "User logged out successfully"
        );

        // TODO: Add token to blacklist (Redis)

        Ok(())
    }

    /// Logout from all devices (revoke all refresh tokens)
    ///
    /// # Future Feature
    /// Requires token storage/blacklist implementation
    #[allow(dead_code)]
    pub async fn logout_all_devices(&self, user_id: Uuid) -> Result<(), AuthApplicationError> {
        info!(user_id = %user_id, "Logout from all devices requested");

        // TODO: Invalidate all refresh tokens for user
        // - Query token store for all user's tokens
        // - Add all to blacklist
        // - Or: Increment user's token_version in DB

        todo!("Logout from all devices not yet implemented")
    }
}

// ============================================
// PRIVATE HELPERS - USER OPERATIONS
// ============================================

impl<AR: AuthUserRepository, PS: PasswordService> AuthService<AR, PS> {
    /// Find user by email
    async fn find_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, AuthApplicationError> {
        self.user_repository
            .find_by_email(email)
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|e| self.db_error("find user by email", email, e))
    }

    /// Find user by ID
    async fn find_user_by_id(&self, id: Uuid) -> Result<User, AuthApplicationError> {
        self.user_repository
            .find_by_id(id)
            .await
            .map_err(|e| self.db_error("find user by id", &id.to_string(), e))?
            .map(Into::into)
            .ok_or_else(|| {
                warn!(user_id = %id, "User not found");
                AuthApplicationError::UserNotFound(id)
            })
    }

    /// Find user by username (not used currently, kept for future)
    #[allow(dead_code)]
    async fn find_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, AuthApplicationError> {
        self.user_repository
            .find_by_username(username)
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|e| self.db_error("find user by username", username, e))
    }

    /// Save user to database
    async fn save_user(&self, user: &User) -> Result<(), AuthApplicationError> {
        self.user_repository
            .save(user)
            .await
            .map_err(|e| {
                error!(
                    error = ?e,
                    user_id = %user.id(),
                    email = %user.email(),
                    "Database error while saving user"
                );
                AuthApplicationError::Internal(format!("Failed to save user: {}", e))
            })
    }

    /// Check if email is available
    async fn is_email_available(&self, email: &str) -> Result<bool, AuthApplicationError> {
        Ok(self.find_user_by_email(email).await?.is_none())
    }
}

// ============================================
// PRIVATE HELPERS - PASSWORD & TOKEN
// ============================================

impl<AR: AuthUserRepository, PS: PasswordService> AuthService<AR, PS> {
    /// Hash password using password service
    fn hash_password(&self, password: &str) -> Result<PasswordHashVO, AuthApplicationError> {
        self.password_service
            .hash_password(password)
            .map_err(|e| {
                error!(error = %e, "Password hashing failed");
                AuthApplicationError::PasswordHashingFailed
            })
    }

    /// Verify password against hash
    fn verify_password(
        &self,
        password: &str,
        password_hash: &PasswordHashVO,
    ) -> Result<(), AuthApplicationError> {
        let is_valid = self
            .password_service
            .verify_password(password, password_hash)
            .map_err(|e| {
                error!(error = %e, "Password verification failed");
                AuthApplicationError::Internal("Password verification failed".to_string())
            })?;

        if !is_valid {
            warn!("Invalid password provided");
            return Err(AuthApplicationError::InvalidCredentials);
        }

        Ok(())
    }

    /// Verify refresh token and extract claims
    fn verify_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<common::jwt::Claims, AuthApplicationError> {
        self.jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token");
                AuthApplicationError::InvalidToken
            })
    }

    /// Ensure user account is active
    fn ensure_user_active(&self, user: &User) -> Result<(), AuthApplicationError> {
        user.ensure_active().map_err(|_| {
            warn!(user_id = %user.id(), "Attempt to access inactive account");
            AuthApplicationError::AccountDeactivated
        })
    }
}

// ============================================
// PRIVATE HELPERS - RESPONSE BUILDING
// ============================================

impl<AR: AuthUserRepository, PS: PasswordService> AuthService<AR, PS> {
    /// Create full auth response with access + refresh tokens
    fn create_auth_response(&self, user: &User) -> Result<AuthResponse, AuthApplicationError> {
        let tokens = self.create_token_pair(user)?;

        Ok(AuthResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: (ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60) as usize, // Convert to seconds
            user: user.into(),
        })
    }

    /// Create refresh response with new tokens
    fn create_refresh_response(
        &self,
        user: &User,
    ) -> Result<RefreshTokenResponse, AuthApplicationError> {
        let tokens = self.create_token_pair(user)?;

        Ok(RefreshTokenResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: (ACCESS_TOKEN_EXPIRY_HOURS * 60 * 60) as usize,
        })
    }

    /// Create access and refresh token pair
    fn create_token_pair(&self, user: &User) -> Result<TokenPair, AuthApplicationError> {
        let role = user.role().as_str().to_string();
        let email = user.email().to_string();
        let user_id = user.id();

        let access_token = self
            .jwt_manager
            .create_user_token(
                user_id,
                email.clone(),
                role.clone(),
                ACCESS_TOKEN_EXPIRY_HOURS as i64,
            )
            .map_err(|e| {
                error!(error = %e, user_id = %user_id, "Failed to create access token");
                AuthApplicationError::TokenGenerationFailed(e)
            })?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(user_id, email, role, REFRESH_TOKEN_EXPIRY_DAYS as i64)
            .map_err(|e| {
                error!(error = %e, user_id = %user_id, "Failed to create refresh token");
                AuthApplicationError::TokenGenerationFailed(e)
            })?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    /// Create database error with context
    fn db_error<E: std::fmt::Debug>(
        &self,
        operation: &str,
        context: &str,
        error: E,
    ) -> AuthApplicationError {
        error!(
            error = ?error,
            operation = %operation,
            context = %context,
            "Database error"
        );
        AuthApplicationError::Internal(format!("Database error during {}: {:?}", operation, error))
    }
}

// ============================================
// HELPER TYPES
// ============================================

/// Internal helper for token pair
struct TokenPair {
    access_token: String,
    refresh_token: String,
}

// ============================================
// FROM TRAIT IMPLEMENTATIONS
// ============================================

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().to_string(),
            email: user.email().to_string(),
            display_name: user.display_name().to_string(),
            username: user.username().to_string(),
            role: user.role().as_str().to_string(),
            is_active: user.is_active(),
            email_verified: user.is_email_verified(),
        }
    }
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add comprehensive tests
    // - Test registration flow
    // - Test login with valid credentials
    // - Test login with invalid credentials
    // - Test token refresh
    // - Test logout
    // - Test inactive account rejection
}