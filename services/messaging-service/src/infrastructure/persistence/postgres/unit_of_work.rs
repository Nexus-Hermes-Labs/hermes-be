// The MutexGuard is intentionally held across `.await` — we need exclusive
// access to the SQLx transaction for the full duration of each query.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::conversation::{Conversation, ConversationMember};
use crate::application::ports::unit_of_work::{MessagingUnitOfWork, MessagingUnitOfWorkFactory};
use crate::domain::unit_of_work::{ConversationMemberWriter, ConversationWriter};

// ─────────────────────────────────────────────────────────────────────────────
// Shared transaction handle
// ─────────────────────────────────────────────────────────────────────────────

type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

fn tx_consumed_err() -> RepositoryError {
    RepositoryError::Mapping("transaction already consumed".into())
}

// ─────────────────────────────────────────────────────────────────────────────
// PgConversationWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgConversationWriter {
    tx: SharedTx,
}

#[async_trait]
impl ConversationWriter for PgConversationWriter {
    async fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            "INSERT INTO conversations (id, type, created_at) VALUES ($1, $2::conversation_type, $3)",
        )
        .bind(conversation.id())
        .bind(conversation.conversation_type().as_str())
        .bind(conversation.created_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgConversationMemberWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgConversationMemberWriter {
    tx: SharedTx,
}

#[async_trait]
impl ConversationMemberWriter for PgConversationMemberWriter {
    async fn save_batch(&self, members: &[ConversationMember]) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        for m in members {
            sqlx::query(
                "INSERT INTO conversation_members (conversation_id, user_id, joined_at) \
                 VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            )
            .bind(m.conversation_id())
            .bind(m.user_id())
            .bind(m.joined_at())
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgMessagingUnitOfWork
// ─────────────────────────────────────────────────────────────────────────────

/// `SQLx`-backed [`MessagingUnitOfWork`].
///
/// Both sub-writers share the same transaction via [`SharedTx`].
/// Dropping without committing rolls the transaction back automatically.
pub struct PgMessagingUnitOfWork {
    tx: SharedTx,
    conversations: PgConversationWriter,
    members: PgConversationMemberWriter,
}

impl PgMessagingUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared = Arc::new(Mutex::new(Some(tx)));
        Self {
            conversations: PgConversationWriter { tx: shared.clone() },
            members: PgConversationMemberWriter { tx: shared.clone() },
            tx: shared,
        }
    }
}

impl std::fmt::Debug for PgMessagingUnitOfWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgMessagingUnitOfWork")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl MessagingUnitOfWork for PgMessagingUnitOfWork {
    fn conversations(&self) -> &dyn ConversationWriter {
        &self.conversations
    }

    fn members(&self) -> &dyn ConversationMemberWriter {
        &self.members
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
// PgMessagingUnitOfWorkFactory
// ─────────────────────────────────────────────────────────────────────────────

/// Creates [`PgMessagingUnitOfWork`] by opening a new `SQLx` transaction.
#[derive(Debug)]
pub struct PgMessagingUnitOfWorkFactory {
    pool: PgPool,
}

impl PgMessagingUnitOfWorkFactory {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessagingUnitOfWorkFactory for PgMessagingUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn MessagingUnitOfWork>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgMessagingUnitOfWork::new(tx)))
    }
}
