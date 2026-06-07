use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use super::{OAuthAccount, OAuthProvider};

/// Read access for the `OAuthAccount` aggregate.
///
/// Writes happen inside the auth Unit of Work (see
/// [`OAuthAccountWriter`](crate::domain::unit_of_work::OAuthAccountWriter)) so
/// that linking an account and creating its credential/session commit
/// atomically. This trait only exposes the lookups the login flow needs.
#[async_trait]
pub trait OAuthAccountRepository: Send + Sync {
    /// Find the link for a given provider + external subject id, if any.
    async fn find_by_provider_and_subject(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, RepositoryError>;
}
