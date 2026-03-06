use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::unit_of_work::ChannelWriter;

/// Unit of Work that coordinates atomic writes to multiple channels.
///
/// Dropping without calling `commit` rolls the transaction back automatically.
#[async_trait]
pub trait ChannelUnitOfWork: Send {
    fn channels(&self) -> &dyn ChannelWriter;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}

/// Factory that opens a new [`ChannelUnitOfWork`] transaction.
#[async_trait]
pub trait ChannelUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn ChannelUnitOfWork>, RepositoryError>;
}
