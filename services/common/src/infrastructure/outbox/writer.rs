// The MutexGuard is intentionally held across `.await` — we need exclusive
// access to the SQLx transaction for the full duration of each query.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::infrastructure::persistence::error::RepositoryError;

/// Shared handle to a SQLx transaction, used by every per-aggregate writer that
/// participates in a Unit of Work. The `Option` lets `commit`/`rollback`
/// consume the transaction without consuming the parent UoW struct.
pub type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

pub fn new_shared_tx(tx: Transaction<'static, Postgres>) -> SharedTx {
    Arc::new(Mutex::new(Some(tx)))
}

pub fn tx_consumed_err() -> RepositoryError {
    RepositoryError::Mapping("transaction already consumed".into())
}

#[derive(Debug, Clone)]
pub struct NewOutboxEvent {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: Value,
}

impl NewOutboxEvent {
    pub fn new(
        aggregate_id: Uuid,
        aggregate_type: impl Into<String>,
        event_type: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: aggregate_type.into(),
            event_type: event_type.into(),
            payload,
        }
    }
}

/// Transactional writer for the `outbox_events` table. Every service that
/// publishes events owns its own `outbox_events` table; this trait is the
/// single seam through which application services enqueue events inside a
/// running transaction.
#[async_trait]
pub trait OutboxWriter: Send + Sync {
    async fn save(&self, event: &NewOutboxEvent) -> Result<(), RepositoryError>;
}

pub struct PgOutboxWriter {
    tx: SharedTx,
    source_service: String,
}

impl PgOutboxWriter {
    pub fn new(tx: SharedTx, source_service: impl Into<String>) -> Self {
        Self {
            tx,
            source_service: source_service.into(),
        }
    }
}

#[async_trait]
impl OutboxWriter for PgOutboxWriter {
    async fn save(&self, event: &NewOutboxEvent) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r#"
            INSERT INTO outbox_events (
                id, aggregate_id, aggregate_type, event_type, payload,
                source_service, status, retry_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', 0)
            "#,
        )
        .bind(event.id)
        .bind(event.aggregate_id)
        .bind(&event.aggregate_type)
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(&self.source_service)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}
