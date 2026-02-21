// services/auth-service/src/presentation/http/dto/verify.rs
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// Verification tokens are alphanumeric + safe URL chars (no null bytes or control chars)
static TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._\-]+$").expect("Failed to compile token regex"));

#[derive(Debug, Clone, Deserialize, Validate, IntoParams)]
pub struct VerifyEmailRequest {
    #[validate(
        length(min = 1, max = 512),
        regex(path = *TOKEN_REGEX, message = "Invalid value")
    )]
    #[param(min_length = 1, max_length = 512, pattern = r"^[a-zA-Z0-9._\-]+$")]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub message: String,
}
