use async_trait::async_trait;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::unit_of_work::{GuildInviteWriter, GuildMemberWriter, GuildWriter};

/// Transactional unit of work for cross-aggregate guild operations.
///
/// Exposes scoped write repositories for each aggregate. All writes share the
/// same underlying database transaction. Call [`commit`](GuildUnitOfWork::commit)
/// to persist atomically, or drop the value to roll back.
///
/// The `SQLx` transaction is hidden entirely inside the infrastructure
/// implementation — no persistence types appear in this trait.
#[async_trait]
pub trait GuildUnitOfWork: Send {
    fn guilds(&self) -> &dyn GuildWriter;
    fn members(&self) -> &dyn GuildMemberWriter;
    fn invites(&self) -> &dyn GuildInviteWriter;

    /// Commit the transaction, making all writes durable.
    ///
    /// Consumes `self` so the unit of work cannot be used after committing.
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}

/// Opens a new database transaction and wraps it in a [`GuildUnitOfWork`].
#[async_trait]
pub trait GuildUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn GuildUnitOfWork>, RepositoryError>;
}
