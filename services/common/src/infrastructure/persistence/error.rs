use thiserror::Error;

// TODO: REMOVE. will be service specific
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    /// DB row → domain model conversion failed (e.g. unknown enum value)
    #[error("Mapping error: {0}")]
    Mapping(String),
}

impl RepositoryError {
    /// Convenience constructor — matches Auth Service's usage pattern
    pub fn not_found<T: std::fmt::Display>(entity: &str, id: T) -> Self {
        Self::NotFound(format!("{} with id '{}' not found", entity, id))
    }
}