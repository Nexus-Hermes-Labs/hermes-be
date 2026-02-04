use crate::domain::user_relationship::erorr::UserRelationshipDomainError;

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    Friend,
    Blocked,
    PendingIncoming,
    PendingOutgoing,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Friend => "friend",
            RelationshipType::Blocked => "blocked",
            RelationshipType::PendingIncoming => "pending_incoming",
            RelationshipType::PendingOutgoing => "pending_outgoing",
        }
    }
}

impl TryFrom<&str> for RelationshipType {
    type Error = UserRelationshipDomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "friend" => RelationshipType::Friend,
            "blocked" => RelationshipType::Blocked,
            "pending_incoming" => RelationshipType::PendingIncoming,
            "pending_outgoing" => RelationshipType::PendingOutgoing,
            _ => return Err(UserRelationshipDomainError::InvalidRelationshipType(value.into())),
        })
    }
}
