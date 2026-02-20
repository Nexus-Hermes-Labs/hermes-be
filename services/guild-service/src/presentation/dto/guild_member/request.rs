use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Request body for kicking a member from a guild.
#[derive(Debug, Deserialize, ToSchema)]
pub struct KickMemberRequest {
    /// Human-readable reason for the kick (shown in audit logs).
    pub reason: Option<String>,
}

/// Request body for assigning a role to a guild member.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignRoleRequest {
    /// ID of the role to assign.
    pub role_id: Uuid,
}
