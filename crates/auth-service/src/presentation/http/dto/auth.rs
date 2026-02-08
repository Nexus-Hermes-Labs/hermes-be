use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================================================
// AUTHENTICATION ENDPOINTS
// ============================================================================

// ============================================
// Endpoint: POST /api/auth/register
// Purpose:  Register a new user account
// Auth:     Public (no authentication required)
// ============================================

/// Request body for user registration
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    /// User's email address (must be unique)
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// Unique username (3-50 characters)
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,

    /// User's display name (3-100 characters)
    pub display_name: String,

    /// Password (minimum 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Response type: AuthResponse (defined below)
/// Example:
/// ```json
/// {
///   "access_token": "eyJ...",
///   "refresh_token": "eyJ...",
///   "user": { "id": "...", "email": "..." }
/// }
/// ```

// ============================================
// Endpoint: POST /api/auth/login
// Purpose:  Authenticate user with email and password
// Auth:     Public (no authentication required)
// ============================================

/// Request body for user login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// User's email address
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// User's password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Response type: AuthResponse (defined below)

// ============================================
// Endpoint: POST /api/auth/refresh
// Purpose:  Refresh access token using refresh token
// Auth:     Public (refresh token in request body)
// ============================================

/// Request body for token refresh
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    /// Valid refresh token received from login/register
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// Response body for token refresh
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New access token (short-lived, ~1 hour)
    pub access_token: String,

    /// New refresh token (long-lived, ~30 days)
    /// Note: Implements refresh token rotation for security
    pub refresh_token: String,

    /// Expiration time of JWT tokens (in seconds)
    pub expires_in: usize,
}

// ============================================
// POST /api/auth/logout
// Standard: OAuth 2.0 Token Revocation (RFC 7009)
// Auth: Public (no Bearer token required)
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    /// Refresh token to revoke (required)
    /// This is the token received during login/register
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

// ============================================================================
// SHARED TYPES
// ============================================================================

// ============================================
// Shared Response: Authentication Success
// Used by: POST /auth/register, POST /auth/login
// ============================================

/// Standard authentication response with tokens and user info
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// JWT access token for API requests (expires in ~1 hour)
    pub access_token: String,

    /// JWT refresh token for obtaining new access tokens (expires in ~30 days)
    pub refresh_token: String,

    /// Expiration time of JWT tokens (in seconds)
    pub expires_in: usize,

    /// Authenticated user information
    pub user: UserResponse,
}

/// User information included in authentication responses
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User's unique identifier (UUID)
    pub id: String,

    /// User's email address
    pub email: String,

    /// User's display name (e.g., "")
    pub display_name: String,

    /// User's username
    pub username: String,

    /// User's role (e.g., "customer", "vendor", "admin")
    pub role: String,

    /// Whether the user account is active
    pub is_active: bool,

    /// Whether the user's email has been verified
    pub email_verified: bool,
}
