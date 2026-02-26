use tracing::warn;
use uuid::Uuid;

use crate::domain::message::Message;
use crate::domain::reaction::Reaction;
use crate::domain::conversation::Conversation;

/// Fire-and-forget NATS event publisher.
///
/// All methods log on failure but do **not** propagate errors to callers —
/// a NATS hiccup should never roll back a user-visible operation.
#[derive(Debug, Clone)]
pub struct NatsPublisher {
    client: async_nats::Client,
}

impl NatsPublisher {
    #[must_use]
    pub const fn new(client: async_nats::Client) -> Self {
        Self { client }
    }

    // ── Messages ─────────────────────────────────────────────────────────

    pub async fn publish_message_created(&self, msg: &Message) {
        let payload = serde_json::json!({
            "message_id":      msg.id(),
            "channel_id":      msg.channel_id(),
            "conversation_id": msg.conversation_id(),
            "user_id":         msg.user_id(),
            "content":         msg.content().as_str(),
            "created_at":      msg.created_at(),
        });
        self.publish("hermes.message.created", &payload).await;
    }

    pub async fn publish_message_updated(&self, msg: &Message) {
        let payload = serde_json::json!({
            "message_id":      msg.id(),
            "channel_id":      msg.channel_id(),
            "conversation_id": msg.conversation_id(),
            "content":         msg.content().as_str(),
            "edited_at":       msg.edited_at(),
        });
        self.publish("hermes.message.updated", &payload).await;
    }

    pub async fn publish_message_deleted(&self, msg: &Message) {
        let payload = serde_json::json!({
            "message_id":      msg.id(),
            "channel_id":      msg.channel_id(),
            "conversation_id": msg.conversation_id(),
            "user_id":         msg.user_id(),
        });
        self.publish("hermes.message.deleted", &payload).await;
    }

    // ── Reactions ─────────────────────────────────────────────────────────

    pub async fn publish_reaction_added(&self, reaction: &Reaction) {
        let payload = serde_json::json!({
            "message_id": reaction.message_id(),
            "user_id":    reaction.user_id(),
            "emoji":      reaction.emoji().as_str(),
        });
        self.publish("hermes.reaction.added", &payload).await;
    }

    pub async fn publish_reaction_removed(&self, message_id: Uuid, user_id: Uuid, emoji: &str) {
        let payload = serde_json::json!({
            "message_id": message_id,
            "user_id":    user_id,
            "emoji":      emoji,
        });
        self.publish("hermes.reaction.removed", &payload).await;
    }

    // ── Conversations ─────────────────────────────────────────────────────

    pub async fn publish_conversation_created(&self, conv: &Conversation, member_ids: &[Uuid]) {
        let payload = serde_json::json!({
            "conversation_id":   conv.id(),
            "conversation_type": conv.conversation_type().as_str(),
            "member_ids":        member_ids,
            "created_at":        conv.created_at(),
        });
        self.publish("hermes.conversation.created", &payload).await;
    }

    pub async fn publish_conversation_member_joined(&self, conversation_id: Uuid, user_id: Uuid) {
        let payload = serde_json::json!({
            "conversation_id": conversation_id,
            "user_id":         user_id,
        });
        self.publish("hermes.conversation.member.joined", &payload).await;
    }

    // ── Internal ──────────────────────────────────────────────────────────

    async fn publish(&self, subject: &str, payload: &serde_json::Value) {
        let bytes = match serde_json::to_vec(payload) {
            Ok(b)  => b,
            Err(e) => { warn!("NATS serialize error on {subject}: {e}"); return; }
        };
        if let Err(e) = self.client.publish(subject.to_string(), bytes.into()).await {
            warn!("NATS publish error on {subject}: {e}");
        }
    }
}
