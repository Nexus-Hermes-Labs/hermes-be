use serde::Deserialize;
use utoipa::ToSchema;

/// Request body for creating a guild invite link.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    /// Maximum number of times this invite can be used (`None` = unlimited).
    pub max_uses: Option<i32>,
    /// Lifetime of the invite in seconds from the time of creation (`None` = never expires).
    pub max_age_seconds: Option<i64>,
}
