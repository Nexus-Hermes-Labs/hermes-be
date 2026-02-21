use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct RelationshipRequest {
    pub target_user_id: Uuid,
    #[serde(default)]
    #[validate(length(max = 200))]
    #[schema(max_length = 200)]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct RelationshipResponse {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub r#type: String,
    pub message: Option<String>,
}

#[derive(Deserialize, ToSchema, IntoParams, Validate)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schema(minimum = 1, maximum = 100, format = "int64")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: i64,
    #[serde(default)]
    #[validate(range(min = 0))]
    #[schema(minimum = 0, format = "int64")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}
