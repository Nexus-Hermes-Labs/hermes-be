#![allow(clippy::significant_drop_tightening)]

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::outbox::{
    new_shared_tx, tx_consumed_err, OutboxWriter, PgOutboxWriter, SharedTx,
};
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::unit_of_work::UnitOfWork;

use crate::application::ports::unit_of_work::{MessagingUnitOfWork, MessagingUnitOfWorkFactory};
use crate::domain::conversation::{Conversation, ConversationMember};
use crate::domain::message::Message;
use crate::domain::reaction::Reaction;
use crate::domain::unit_of_work::{
    ConversationMemberWriter, ConversationWriter, MessageWriter, ReactionWriter,
};

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

struct PgMessageWriter {
    tx: SharedTx,
}

#[async_trait]
impl MessageWriter for PgMessageWriter {
    async fn save(&self, m: &Message) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO messages
                (id, channel_id, conversation_id, user_id, content, type,
                 reply_to_id, is_deleted, edited_at, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6::message_type,
                 $7, $8, $9, $10, $11)
            ",
        )
        .bind(m.id())
        .bind(m.channel_id())
        .bind(m.conversation_id())
        .bind(m.user_id())
        .bind(m.content().as_str())
        .bind(m.message_type().as_str())
        .bind(m.reply_to_id())
        .bind(m.is_deleted())
        .bind(m.edited_at())
        .bind(m.created_at())
        .bind(m.updated_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn update(&self, m: &Message) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            UPDATE messages
            SET content    = $2,
                is_deleted = $3,
                edited_at  = $4,
                updated_at = $5
            WHERE id = $1
            ",
        )
        .bind(m.id())
        .bind(m.content().as_str())
        .bind(m.is_deleted())
        .bind(m.edited_at())
        .bind(m.updated_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

struct PgReactionWriter {
    tx: SharedTx,
}

#[async_trait]
impl ReactionWriter for PgReactionWriter {
    async fn save(&self, r: &Reaction) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            "INSERT INTO reactions (id, message_id, user_id, emoji, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(r.id())
        .bind(r.message_id())
        .bind(r.user_id())
        .bind(r.emoji().as_str())
        .bind(r.created_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn delete_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<u64, RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        let result = sqlx::query(
            "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&mut **tx)
        .await?;
        Ok(result.rows_affected())
    }
}

pub struct PgMessagingUnitOfWork {
    tx: SharedTx,
    conversations: PgConversationWriter,
    members: PgConversationMemberWriter,
    messages: PgMessageWriter,
    reactions: PgReactionWriter,
    outbox: PgOutboxWriter,
}

impl PgMessagingUnitOfWork {
    fn new(tx: sqlx::Transaction<'static, sqlx::Postgres>, source_service: &str) -> Self {
        let shared = new_shared_tx(tx);
        Self {
            conversations: PgConversationWriter { tx: shared.clone() },
            members: PgConversationMemberWriter { tx: shared.clone() },
            messages: PgMessageWriter { tx: shared.clone() },
            reactions: PgReactionWriter { tx: shared.clone() },
            outbox: PgOutboxWriter::new(shared.clone(), source_service),
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
impl UnitOfWork for PgMessagingUnitOfWork {
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
        }
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.rollback().await?;
        }
        Ok(())
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
    fn messages(&self) -> &dyn MessageWriter {
        &self.messages
    }
    fn reactions(&self) -> &dyn ReactionWriter {
        &self.reactions
    }
    fn outbox(&self) -> &dyn OutboxWriter {
        &self.outbox
    }
}

#[derive(Debug)]
pub struct PgMessagingUnitOfWorkFactory {
    pool: PgPool,
    source_service: String,
}

impl PgMessagingUnitOfWorkFactory {
    pub fn new(pool: PgPool, source_service: impl Into<String>) -> Self {
        Self {
            pool,
            source_service: source_service.into(),
        }
    }
}

#[async_trait]
impl MessagingUnitOfWorkFactory for PgMessagingUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn MessagingUnitOfWork>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgMessagingUnitOfWork::new(
            tx,
            &self.source_service,
        )))
    }
}
