use common::infrastructure::persistence::error::RepositoryError;
use thiserror::Error;

use crate::application::ports::oauth_provider::OAuthProviderError;
use crate::application::ports::oauth_state_store::OAuthStateStoreError;

/// Application-level errors for the OAuth login flow.
#[derive(Debug, Error)]
pub enum OAuthApplicationError {
    /// The requested provider has no credentials configured.
    #[error("OAuth provider is not configured")]
    ProviderNotConfigured,

    /// The CSRF `state` was missing, expired, or already consumed.
    #[error("Invalid or expired OAuth state")]
    InvalidState,

    /// The provider rejected the exchange or returned bad data.
    #[error("OAuth provider error: {0}")]
    ProviderError(String),

    /// The provider did not report the email as verified.
    #[error("Email not verified by provider")]
    EmailNotVerifiedByProvider,

    /// The provider returned an email we can't accept.
    #[error("Invalid email from provider: {0}")]
    InvalidEmail(String),

    /// The matched local account is suspended.
    #[error("Account is suspended")]
    AccountSuspended,

    /// The matched local account is deleted.
    #[error("Account is deleted")]
    AccountDeleted,

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<RepositoryError> for OAuthApplicationError {
    fn from(error: RepositoryError) -> Self {
        OAuthApplicationError::RepositoryError(error.to_string())
    }
}

impl From<OAuthProviderError> for OAuthApplicationError {
    fn from(error: OAuthProviderError) -> Self {
        match error {
            OAuthProviderError::NotConfigured => OAuthApplicationError::ProviderNotConfigured,
            OAuthProviderError::Request(msg) | OAuthProviderError::InvalidResponse(msg) => {
                OAuthApplicationError::ProviderError(msg)
            }
        }
    }
}

impl From<OAuthStateStoreError> for OAuthApplicationError {
    fn from(error: OAuthStateStoreError) -> Self {
        OAuthApplicationError::Internal(error.to_string())
    }
}
