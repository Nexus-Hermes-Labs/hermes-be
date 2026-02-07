use thiserror::Error;
use uuid::Uuid;
use common::persistance::error::RepositoryError;
use crate::domain::user_relationship::erorr::UserRelationshipDomainError;

#[derive(Debug, Error)]
pub enum UserRelationshipApplicationError {
    // =====================================================
    // NOT FOUND ERRORS (Application-specific)
    // =====================================================

    #[error("User not found with ID: {0}")]
    UserNotFound(Uuid),

    #[error("Target user not found: {0}")]
    TargetUserNotFound(String),

    #[error("Relationship not found between users")]
    RelationshipNotFound,

    #[error("Friend request not found")]
    FriendRequestNotFound,

    #[error("Friendship not found between these users")]
    FriendshipNotFound,

    // =====================================================
    // COORDINATION ERRORS (Application-specific)
    // =====================================================

    #[error("Failed to coordinate user and relationship operations")]
    CoordinationError(String),

    // =====================================================
    // WRAPPED ERRORS
    // Domain ve Repository errors
    // =====================================================

    #[error("Domain error: {0}")]
    Domain(#[from] UserRelationshipDomainError),

    // NOT: #[from] kaldırıldı - manuel From impl kullanacağız
    #[error("Repository error: {0}")]
    Repository(RepositoryError),

    // =====================================================
    // INTERNAL ERRORS
    // =====================================================

    #[error("Internal error: {0}")]
    Internal(String),
}

// =====================================================
// REPOSITORY ERROR CONVERSION - Manuel Implementation
// =====================================================

impl From<RepositoryError> for UserRelationshipApplicationError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => {
                if msg.to_lowercase().contains("relationship") {
                    Self::RelationshipNotFound
                }
                else if msg.to_lowercase().contains("user") {
                    Self::Internal(format!("User not found: {}", msg))
                }
                else {
                    Self::Internal(format!("Not found: {}", msg))
                }
            }

            RepositoryError::DuplicateEntry(msg) => {
                // Relationship duplicate ise AlreadyFriends
                if msg.to_lowercase().contains("relationship") {
                    Self::Domain(UserRelationshipDomainError::AlreadyFriends)
                } else {
                    Self::Internal(format!("Duplicate entry: {}", msg))
                }
            }

            RepositoryError::Mapping(_) | RepositoryError::Database(_) => {
                Self::Repository(err)
            }
        }
    }
}

// =====================================================
// HTTP MAPPING & HELPER METHODS
// =====================================================

impl UserRelationshipApplicationError {
    /// Convert to HTTP status code
    pub fn status_code(&self) -> u16 {
        match self {
            // 404 Not Found
            Self::UserNotFound(_)
            | Self::TargetUserNotFound(_)
            | Self::RelationshipNotFound
            | Self::FriendRequestNotFound
            | Self::FriendshipNotFound => 404,

            // Domain errors - delegate to domain error
            Self::Domain(e) if e.is_authorization_error() => 403,
            Self::Domain(e) if e.is_validation_error() => 400,
            Self::Domain(e) if e.is_privacy_error() => 403,
            Self::Domain(e) if e.is_state_error() => 409, // Conflict
            Self::Domain(e) if e.is_business_rule_violation() => 400,
            Self::Domain(_) => 400, // Default for domain errors

            // 500 Internal Server Error
            Self::Repository(_) | Self::Internal(_) | Self::CoordinationError(_) => 500,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            // Application-specific messages
            Self::UserNotFound(_) => "User not found".to_string(),
            Self::TargetUserNotFound(username) => {
                format!("User '{}' not found", username)
            }
            Self::RelationshipNotFound => "Relationship not found".to_string(),
            Self::FriendRequestNotFound => "Friend request not found".to_string(),
            Self::FriendshipNotFound => "You are not friends with this user".to_string(),

            // Domain errors - pass through
            Self::Domain(e) => e.to_string(),

            // Hide internal errors
            Self::Repository(_) | Self::Internal(_) | Self::CoordinationError(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }

    /// Check if error should be logged
    pub fn should_log(&self) -> bool {
        matches!(
            self,
            Self::Repository(_) | Self::Internal(_) | Self::CoordinationError(_)
        )
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::UserNotFound(_) => "USER_NOT_FOUND",
            Self::TargetUserNotFound(_) => "TARGET_USER_NOT_FOUND",
            Self::RelationshipNotFound => "RELATIONSHIP_NOT_FOUND",
            Self::FriendRequestNotFound => "FRIEND_REQUEST_NOT_FOUND",
            Self::FriendshipNotFound => "FRIENDSHIP_NOT_FOUND",

            // Domain errors - use domain error code
            Self::Domain(e) => e.as_str(),

            Self::Repository(_) => "REPOSITORY_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::CoordinationError(_) => "COORDINATION_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::user_relationship::erorr::UserRelationshipDomainError;
    use super::*;

    #[test]
    fn test_not_found_errors() {
        let err = UserRelationshipApplicationError::UserNotFound(uuid::Uuid::new_v4());
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.error_code(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_domain_error_delegation() {
        let domain_err = UserRelationshipDomainError::AlreadyFriends;
        let app_err = UserRelationshipApplicationError::Domain(domain_err);

        assert_eq!(app_err.status_code(), 409); // Conflict
        assert!(app_err.user_message().contains("already friends"));
    }

    #[test]
    fn test_domain_error_code_passthrough() {
        let domain_err = UserRelationshipDomainError::MessageTooLong;
        let app_err = UserRelationshipApplicationError::Domain(domain_err);

        assert_eq!(app_err.error_code(), "message_too_long");
    }

    #[test]
    fn test_repository_not_found_conversion() {
        let repo_err = RepositoryError::NotFound("Relationship with id '123' not found".to_string());
        let app_err: UserRelationshipApplicationError = repo_err.into();

        assert!(matches!(app_err, UserRelationshipApplicationError::RelationshipNotFound));
        assert_eq!(app_err.status_code(), 404);
    }

    #[test]
    fn test_repository_duplicate_conversion() {
        let repo_err = RepositoryError::DuplicateEntry("Relationship already exists".to_string());
        let app_err: UserRelationshipApplicationError = repo_err.into();

        assert!(matches!(
            app_err,
            UserRelationshipApplicationError::Domain(UserRelationshipDomainError::AlreadyFriends)
        ));
        assert_eq!(app_err.status_code(), 409);
    }
}