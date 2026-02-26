use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// POST /conversations/dm — open or retrieve a DM
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct OpenDmRequest {
    pub target_user_id: Uuid,
}

/// POST /conversations/group — create a group DM
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGroupDmRequest {
    /// 2–9 other user IDs (requester is added automatically — total 3–10).
    #[validate(length(min = 2, max = 9))]
    pub member_ids: Vec<Uuid>,
}

/// POST /conversations/:id/members — add a user to a group DM
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}
