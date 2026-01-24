use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Concurrency error: entity was modified by another transaction")]
    ConcurrencyError,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl RepositoryError {
    pub fn not_found(entity_type: impl Into<String>, id: impl ToString) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.to_string(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::DuplicateEntry(_))
    }
}

impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound { entity_type, .. } => AppError::NotFound { entity_type },
            RepositoryError::DuplicateEntry(msg) => AppError::Conflict(msg),
            RepositoryError::Database(e) => AppError::Database(e.to_string()),
            _ => AppError::InternalServerError(anyhow::anyhow!("{}", err).to_string()),
        }
    }
}
