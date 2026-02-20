use crate::domain::auth_credential::AuthCredentialRepository;
use async_trait::async_trait;
use std::sync::Arc;

use crate::application::services::authentication::error::AuthApplicationError;

#[async_trait]
pub trait ClearExpiredTokens: Send + Sync {
    async fn execute(&self) -> Result<u64, AuthApplicationError>;
}

pub struct ClearExpiredVerificationTokens {
    repo: Arc<dyn AuthCredentialRepository>,
}

impl ClearExpiredVerificationTokens {
    pub fn new(repo: Arc<dyn AuthCredentialRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ClearExpiredTokens for ClearExpiredVerificationTokens {
    async fn execute(&self) -> Result<u64, AuthApplicationError> {
        self.repo
            .clear_expired_verification_tokens()
            .await
            .map_err(AuthApplicationError::from)
    }
}
