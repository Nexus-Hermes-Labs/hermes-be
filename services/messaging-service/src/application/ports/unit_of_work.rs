use async_trait::async_trait;

use common::infrastructure::outbox::OutboxWriter;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::unit_of_work::{
    run_in_transaction, UnitOfWork, UowCallback,
};

use crate::domain::unit_of_work::{
    ConversationMemberWriter, ConversationWriter, MessageWriter, ReactionWriter,
};

pub type MessagingTransactionCallback<'a> = UowCallback<'a, dyn MessagingUnitOfWork>;

#[async_trait]
pub trait MessagingUnitOfWork: UnitOfWork {
    fn conversations(&self) -> &dyn ConversationWriter;
    fn members(&self) -> &dyn ConversationMemberWriter;
    fn messages(&self) -> &dyn MessageWriter;
    fn reactions(&self) -> &dyn ReactionWriter;
    fn outbox(&self) -> &dyn OutboxWriter;
}

#[async_trait]
pub trait MessagingUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn MessagingUnitOfWork>, RepositoryError>;

    async fn transaction(
        &self,
        operation: MessagingTransactionCallback<'_>,
    ) -> Result<(), RepositoryError> {
        let uow = self.begin().await?;
        run_in_transaction(uow, operation).await
    }
}
