use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Reaction;

#[derive(Debug, Serialize, ToSchema)]
pub struct ReactionResponse {
    pub id:         Uuid,
    pub message_id: Uuid,
    pub user_id:    Uuid,
    pub emoji:      String,
    pub created_at: DateTime<Utc>,
}

impl From<Reaction> for ReactionResponse {
    fn from(r: Reaction) -> Self {
        Self {
            id:         r.id(),
            message_id: r.message_id(),
            user_id:    r.user_id(),
            emoji:      r.emoji().as_str().to_string(),
            created_at: r.created_at(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReactionCountResponse {
    pub emoji: String,
    pub count: i64,
}
