use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Request body for creating a new channel in a guild
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateChannelRequest {
    /// Channel display name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100, example = "general")]
    pub name: String,

    /// Channel type: "text", "voice", "category", or "announcement"
    #[validate(length(min = 1))]
    #[schema(example = "text")]
    pub channel_type: String,

    /// Optional parent category channel ID
    pub parent_id: Option<Uuid>,

    /// Optional channel topic or description (max 1024 characters)
    #[validate(length(max = 1024))]
    #[schema(max_length = 1024)]
    pub description: Option<String>,
}

/// Request body for partially updating a channel
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateChannelRequest {
    /// New channel name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub name: Option<String>,

    /// New parent category channel ID (set to change parent)
    pub parent_id: Option<Uuid>,

    /// New channel description (max 1024 characters)
    #[validate(length(max = 1024))]
    #[schema(max_length = 1024)]
    pub description: Option<String>,

    /// New display position within the guild
    pub position: Option<i32>,
}
