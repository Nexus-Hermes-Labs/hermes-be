use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, IntoParams, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schema(minimum = 1, maximum = 100)]
    pub limit: i64,
    #[serde(default)]
    #[validate(range(min = 0))]
    #[schema(minimum = 0)]
    pub offset: i64,
}

const fn default_limit() -> i64 {
    50
}

/// Request body for kicking a member from a guild.
#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct KickMemberRequest {
    /// Human-readable reason for the kick (shown in audit logs).
    #[validate(custom(function = "common::utils::validator::validate_no_null_bytes"))]
    pub reason: Option<String>,
}

/// Request body for assigning a role to a guild member.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AssignRoleRequest {
    /// ID of the role to assign.
    pub role_id: Uuid,
}
