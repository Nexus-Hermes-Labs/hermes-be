use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// Permissions bitfield
    pub permissions: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// RGB integer (0 = no color)
    pub color: Option<i32>,
    pub permissions: Option<i64>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
}
