use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Request body for creating a guild invite link.
#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateInviteRequest {
    /// Maximum number of times this invite can be used (`None` = unlimited).
    #[validate(range(min = 1))]
    #[schema(minimum = 1)]
    pub max_uses: Option<i32>,
    /// Lifetime of the invite in seconds from the time of creation (`None` = never expires).
    #[validate(range(min = 0))]
    #[schema(minimum = 0)]
    pub max_age_seconds: Option<i64>,
}
