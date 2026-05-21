use async_trait::async_trait;

use common::infrastructure::outbox::OutboxWriter;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::unit_of_work::{
    run_in_transaction, UnitOfWork, UowCallback,
};

use crate::domain::unit_of_work::{MessageWriter, ReactionWriter};

pub type ChatTransactionCallback<'a> = UowCallback<'a, dyn ChatUnitOfWork>;

#[async_trait]
pub trait ChatUnitOfWork: UnitOfWork {
    fn messages(&self) -> &dyn MessageWriter;
    fn reactions(&self) -> &dyn ReactionWriter;
    fn outbox(&self) -> &dyn OutboxWriter;
}

#[async_trait]
pub trait ChatUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn ChatUnitOfWork>, RepositoryError>;

    async fn transaction(
        &self,
        operation: ChatTransactionCallback<'_>,
    ) -> Result<(), RepositoryError> {
        let uow = self.begin().await?;
        run_in_transaction(uow, operation).await
    }
}
