// The MutexGuard is intentionally held across `.await` — we need exclusive
// access to the SQLx transaction for the full duration of each query.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::channel::Channel;
use crate::application::ports::unit_of_work::{ChannelUnitOfWork, ChannelUnitOfWorkFactory};
use crate::domain::unit_of_work::ChannelWriter;

// ─────────────────────────────────────────────────────────────────────────────
// Shared transaction handle
// ─────────────────────────────────────────────────────────────────────────────

type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

fn tx_consumed_err() -> RepositoryError {
    RepositoryError::Mapping("transaction already consumed".into())
}

// ─────────────────────────────────────────────────────────────────────────────
// PgChannelWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgChannelWriter {
    tx: SharedTx,
}

#[async_trait]
impl ChannelWriter for PgChannelWriter {
    async fn save(&self, channel: &Channel) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO channels (
                id, guild_id, parent_id, name, type,
                description, position, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5::channel_type, $6, $7, $8, $9, $10)
            ",
        )
        .bind(channel.id())
        .bind(channel.guild_id())
        .bind(channel.parent_id())
        .bind(channel.name().as_str())
        .bind(channel.channel_type().as_str())
        .bind(channel.description())
        .bind(channel.position())
        .bind(channel.created_at())
        .bind(channel.updated_at())
        .bind(if channel.is_deleted() {
            Some(channel.updated_at())
        } else {
            None
        })
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgChannelUnitOfWork
// ─────────────────────────────────────────────────────────────────────────────

/// `SQLx`-backed [`ChannelUnitOfWork`].
pub struct PgChannelUnitOfWork {
    tx: SharedTx,
    channels: PgChannelWriter,
}

impl PgChannelUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared = Arc::new(Mutex::new(Some(tx)));
        Self {
            channels: PgChannelWriter { tx: shared.clone() },
            tx: shared,
        }
    }
}

impl std::fmt::Debug for PgChannelUnitOfWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgChannelUnitOfWork")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ChannelUnitOfWork for PgChannelUnitOfWork {
    fn channels(&self) -> &dyn ChannelWriter {
        &self.channels
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgChannelUnitOfWorkFactory
// ─────────────────────────────────────────────────────────────────────────────

/// Creates [`PgChannelUnitOfWork`] by opening a new `SQLx` transaction.
#[derive(Debug)]
pub struct PgChannelUnitOfWorkFactory {
    pool: PgPool,
}

impl PgChannelUnitOfWorkFactory {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelUnitOfWorkFactory for PgChannelUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn ChannelUnitOfWork>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgChannelUnitOfWork::new(tx)))
    }
}
