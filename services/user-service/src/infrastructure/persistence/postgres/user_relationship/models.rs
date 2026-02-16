use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::user_relationship::entity::UserRelationship;
use crate::domain::user_relationship::valueobject::RelationshipType;

/// Database model for user_relationships table
#[derive(Debug, Clone, FromRow)]
pub struct UserRelationshipRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_user_id: Uuid,
    pub r#type: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRelationshipRow> for UserRelationship {
    type Error = String;

    fn try_from(row: UserRelationshipRow) -> Result<Self, Self::Error> {
        let relationship_type = row
            .r#type
            .parse::<RelationshipType>()
            .map_err(|e| e.to_string())?;

        Ok(UserRelationship::from_persisted(
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

impl From<&UserRelationship> for UserRelationshipRow {
    fn from(entity: &UserRelationship) -> Self {
        Self {
            id: entity.id(),
            user_id: entity.user_id(),
            target_user_id: entity.target_user_id(),
            r#type: entity.relationship_type().as_str().to_string(),
            message: entity.message().map(String::from),
            created_at: entity.created_at(),
            updated_at: entity.updated_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_to_entity_conversion() {
        let row = UserRelationshipRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            target_user_id: Uuid::new_v4(),
            r#type: "friend".to_string(),
            message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let entity = UserRelationship::try_from(row).unwrap();
        assert_eq!(entity.relationship_type(), RelationshipType::Friend);
        assert!(entity.message().is_none());
    }

    #[test]
    fn test_row_to_entity_with_message() {
        let row = UserRelationshipRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            target_user_id: Uuid::new_v4(),
            r#type: "pending_outgoing".to_string(),
            message: Some("Hello!".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let entity = UserRelationship::try_from(row).unwrap();
        assert_eq!(entity.relationship_type(), RelationshipType::PendingOutgoing);
        assert_eq!(entity.message(), Some("Hello!"));
    }

    #[test]
    fn test_entity_to_row_conversion() {
        let entity =
            UserRelationship::create_request(Uuid::new_v4(), Uuid::new_v4(), "Hi!".to_string())
                .unwrap();

        let row = UserRelationshipRow::from(&entity);
        assert_eq!(row.r#type, "pending_outgoing");
        assert_eq!(row.message, Some("Hi!".to_string()));
    }

    #[test]
    fn test_invalid_type_fails() {
        let row = UserRelationshipRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            target_user_id: Uuid::new_v4(),
            r#type: "invalid".to_string(),
            message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = UserRelationship::try_from(row);
        assert!(result.is_err());
    }
}
