use thiserror::Error;

/// Errors for the `OAuthAccount` aggregate.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OAuthAccountError {
    #[error("Unknown OAuth provider: {0}")]
    UnknownProvider(String),
}
