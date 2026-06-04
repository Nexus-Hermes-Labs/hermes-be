use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================
// FORGOT PASSWORD
// ============================================

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

// ============================================
// RESET PASSWORD (via token)
// ============================================

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
}

// ============================================
// CHANGE PASSWORD (authenticated)
// ============================================

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, max = 128, message = "New password must be 8-128 characters"))]
    pub new_password: String,
}
