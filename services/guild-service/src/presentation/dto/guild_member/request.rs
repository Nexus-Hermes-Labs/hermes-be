use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema)]
pub struct KickMemberRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}
