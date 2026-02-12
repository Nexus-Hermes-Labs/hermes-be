use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenHasherError {
    #[error("Token cannot be empty")]
    EmptyToken,
}
