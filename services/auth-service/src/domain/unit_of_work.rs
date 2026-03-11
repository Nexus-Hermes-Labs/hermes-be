use async_trait::async_trait;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::auth_credential::AuthCredential;
use crate::domain::auth_session::AuthSession;

/// Transactional writer for the `auth_credentials` table.
///
/// All methods execute within the shared transaction owned by
/// [`AuthUnitOfWork`]. Reads and single-aggregate writes stay in
/// [`AuthCredentialRepository`](crate::domain::auth_credential::AuthCredentialRepository).
#[async_trait]
pub trait CredentialWriter: Send + Sync {
    async fn save(&self, credential: &AuthCredential) -> Result<(), RepositoryError>;
    async fn update(&self, credential: &AuthCredential) -> Result<(), RepositoryError>;
    async fn set_verification_token(
        &self,
        credential_id: Uuid,
        token: &str,
        expires_in_hours: i64,
    ) -> Result<(), RepositoryError>;
}

/// Transactional writer for the `auth_sessions` table.
#[async_trait]
pub trait SessionWriter: Send + Sync {
    async fn save(&self, session: &AuthSession) -> Result<(), RepositoryError>;
}
