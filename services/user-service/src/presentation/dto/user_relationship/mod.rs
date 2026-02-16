use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RelationshipRequest {
    pub target_user_id: Uuid,
    #[serde(default)]
    pub message: String,
}

#[derive(Serialize)]
pub struct RelationshipResponse {
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub r#type: String,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}