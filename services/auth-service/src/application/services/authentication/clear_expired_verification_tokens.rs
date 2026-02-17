use std::sync::Arc;
use crate::domain::auth_credential::AuthCredentialRepository;
use crate::application::background::email_verification_cleanup::ClearExpiredVerificationTokensTrait;
use async_trait::async_trait;

use crate::application::services::authentication::error::AuthApplicationError;

pub struct ClearExpiredVerificationTokens {
    repo: Arc<dyn AuthCredentialRepository>,
}

impl ClearExpiredVerificationTokens {
    pub fn new(repo: Arc<dyn AuthCredentialRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ClearExpiredVerificationTokensTrait for ClearExpiredVerificationTokens {
    async fn execute(&self) -> Result<u64, AuthApplicationError> {
        self.repo.clear_expired_verification_tokens().await.map_err(AuthApplicationError::from)
    }
}