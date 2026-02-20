use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Request body for creating a new guild role.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    /// Role display name (1–100 characters, must be unique within the guild).
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// Permissions bitfield (combine `Permissions::*` constants with bitwise OR).
    pub permissions: i64,
}

/// Request body for updating an existing guild role.
///
/// All fields are optional; only the provided fields are changed.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    /// New role display name (1–100 characters).
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// New role color as an RGB integer (0 = no color / default grey).
    pub color: Option<i32>,
    /// New permissions bitfield.
    pub permissions: Option<i64>,
    /// Whether to display this role separately in the member list sidebar.
    pub hoist: Option<bool>,
    /// Whether all members can @mention this role.
    pub mentionable: Option<bool>,
}
