use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::unit_of_work::{ConversationMemberWriter, ConversationWriter};

/// Unit of Work that coordinates atomic writes to conversations + members.
///
/// Dropping without calling `commit` rolls the transaction back automatically.
#[async_trait]
pub trait MessagingUnitOfWork: Send {
    fn conversations(&self) -> &dyn ConversationWriter;
    fn members(&self) -> &dyn ConversationMemberWriter;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}

/// Factory that opens a new [`MessagingUnitOfWork`] transaction.
#[async_trait]
pub trait MessagingUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn MessagingUnitOfWork>, RepositoryError>;
}
