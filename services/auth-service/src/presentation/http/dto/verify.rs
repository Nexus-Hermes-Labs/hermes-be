// services/auth-service/src/presentation/http/dto/verify.rs
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, IntoParams)]
pub struct VerifyEmailRequest {
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub message: String,
}
