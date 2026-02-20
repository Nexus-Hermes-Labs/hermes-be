#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration extraction error: {0}")]
    Extraction(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
