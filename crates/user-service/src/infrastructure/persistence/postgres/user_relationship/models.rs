use chrono::{DateTime, Utc};
use crate::domain::user_relationship::entity::UserRelationship;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
use crate::domain::user_relationship::valueobject::RelationshipType;

/// Flat database row — mirrors the columns UserRelationsihp Service SELECTs.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRelationshipRow {
    pub user_id: uuid::Uuid,
    pub friend_id: uuid::Uuid,
    pub relationship_type: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRelationshipRow> for UserRelationship{
    type Error = UserRelationshipDomainError;

    fn try_from(row: UserRelationshipRow) -> Result<Self, Self::Error> {
        let relationship_type = RelationshipType::try_from(row.relationship_type.as_str())?;
        
        Ok(UserRelationship::reconstruct(
            row.user_id,
            row.friend_id,
            row.friend_id,
            relationship_type,
            row.message,
            row.created_at,
            row.updated_at,
        ))
    }
}
