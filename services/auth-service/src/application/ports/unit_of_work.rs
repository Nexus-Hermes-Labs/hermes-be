use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::unit_of_work::{CredentialWriter, SessionWriter};

/// Unit of Work that coordinates atomic writes to credentials + sessions.
///
/// Dropping without calling `commit` rolls the transaction back automatically.
#[async_trait]
pub trait AuthUnitOfWork: Send {
    fn credentials(&self) -> &dyn CredentialWriter;
    fn sessions(&self) -> &dyn SessionWriter;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}

/// Factory that opens a new [`AuthUnitOfWork`] transaction.
#[async_trait]
pub trait AuthUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn AuthUnitOfWork>, RepositoryError>;
}
