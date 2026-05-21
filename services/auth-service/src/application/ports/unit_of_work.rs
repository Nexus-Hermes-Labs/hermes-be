use async_trait::async_trait;

use common::infrastructure::outbox::OutboxWriter;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::unit_of_work::{
    run_in_transaction, UnitOfWork, UowCallback,
};

use crate::domain::unit_of_work::{CredentialWriter, SessionWriter};

/// Per-call transaction callback for the auth UoW.
pub type AuthTransactionCallback<'a> = UowCallback<'a, dyn AuthUnitOfWork>;

/// Unit of Work that coordinates atomic writes to credentials + sessions +
/// outbox. Inherits [`UnitOfWork`] for commit/rollback.
#[async_trait]
pub trait AuthUnitOfWork: UnitOfWork {
    fn credentials(&self) -> &dyn CredentialWriter;
    fn sessions(&self) -> &dyn SessionWriter;
    fn outbox(&self) -> &dyn OutboxWriter;
}

/// Factory that opens a new [`AuthUnitOfWork`] transaction. The default
/// `transaction()` impl delegates to the shared `run_in_transaction` helper.
#[async_trait]
pub trait AuthUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn AuthUnitOfWork>, RepositoryError>;

    async fn transaction(
        &self,
        operation: AuthTransactionCallback<'_>,
    ) -> Result<(), RepositoryError> {
        let uow = self.begin().await?;
        run_in_transaction(uow, operation).await
    }
}
