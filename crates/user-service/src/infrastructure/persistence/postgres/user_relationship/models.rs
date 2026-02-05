use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::user_relationship::entity::UserRelationship;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user_relationship::valueobject::RelationshipType;

/// Flat database row — mirrors the user_relationships table columns
#[derive(Debug, sqlx::FromRow)]
pub struct UserRelationshipRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub relationship_type: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRelationshipRow> for UserRelationship {
    type Error = UserRelationshipDomainError;

    fn try_from(row: UserRelationshipRow) -> Result<Self, Self::Error> {
        let relationship_type = RelationshipType::try_from(row.relationship_type.as_str())?;

        Ok(UserRelationship::reconstruct(
            row.id,
            row.user_id,
            row.target_user_id,
            relationship_type,
            row.message,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// Helper: Convert domain entity to SQL parameters (for insert/update)
impl UserRelationship {
    pub fn to_row_params(&self) -> (Uuid, Uuid, Uuid, String, Option<String>, DateTime<Utc>, DateTime<Utc>) {
        (
            *self.id(),
            *self.user_id(),
            *self.target_user_id(),
            self.relationship_type().as_str().to_string(),
            self.message().map(|s| s.to_string()),
            *self.created_at(),
            *self.updated_at(),
        )
    }
}