use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

use crate::infrastructure::persistence::error::RepositoryError;

/// Future type returned by a transaction callback. The borrow of the
/// [`UnitOfWork`] must outlive the inner future, hence the HRTB on
/// [`UowCallback`].
pub type UowFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), RepositoryError>> + Send + 'a>>;

/// Boxed closure executed inside [`run_in_transaction`]. Each service exposes a
/// type alias that fixes the trait-object argument (e.g.
/// `UowCallback<'a, dyn AuthUnitOfWork>`).
pub type UowCallback<'a, U> =
    Box<dyn for<'uow> FnOnce(&'uow U) -> UowFuture<'uow> + Send + 'a>;

/// Shared commit/rollback contract for every per-service unit of work.
///
/// Dropping the implementer without calling `commit` rolls the underlying
/// transaction back automatically — `run_in_transaction` relies on that
/// safety net when the operation closure returns early.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError>;
}

/// Run `operation` against `uow` and finalise the transaction. On `Ok` the
/// transaction is committed; on `Err` it is rolled back and the original
/// error is returned. Per-service factory traits delegate their
/// `transaction()` default impl to this helper.
pub async fn run_in_transaction<U: UnitOfWork + ?Sized>(
    uow: Box<U>,
    operation: UowCallback<'_, U>,
) -> Result<(), RepositoryError> {
    if let Err(err) = operation(uow.as_ref()).await {
        uow.rollback().await?;
        return Err(err);
    }
    uow.commit().await
}
