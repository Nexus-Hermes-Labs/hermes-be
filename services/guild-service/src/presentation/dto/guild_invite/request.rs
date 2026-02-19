use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    /// Maximum number of times this invite can be used (None = unlimited)
    pub max_uses: Option<i32>,
    /// Expiry in seconds from now (None = never expires)
    pub max_age_seconds: Option<i64>,
}
