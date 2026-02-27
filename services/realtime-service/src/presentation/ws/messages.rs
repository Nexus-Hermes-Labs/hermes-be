//! WebSocket message types for client ↔ server communication.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Client → Server ──────────────────────────────────────────────────────────

/// Messages sent from a connected WebSocket client to the server.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientMsg {
    /// Subscribe to real-time events for a channel or conversation.
    Subscribe {
        /// UUID of the channel or conversation to subscribe to.
        context_id: Uuid,
    },
    /// Stop receiving events for a context.
    Unsubscribe {
        /// UUID of the channel or conversation to unsubscribe from.
        context_id: Uuid,
    },
    /// Keep-alive ping from the client.
    Heartbeat,
}

// ── Server → Client ──────────────────────────────────────────────────────────

/// Messages sent from the server to a connected WebSocket client.
#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerMsg {
    /// Sent immediately after a successful WebSocket upgrade.
    Hello {
        /// Recommended client heartbeat interval in milliseconds.
        heartbeat_interval: u32,
    },
    /// Acknowledgement of a client `HEARTBEAT`.
    HeartbeatAck,
    /// Confirmation that the client has been subscribed to a context.
    Subscribed {
        /// The context UUID the client is now subscribed to.
        context_id: Uuid,
    },
    /// Confirmation that the client has been unsubscribed from a context.
    Unsubscribed {
        /// The context UUID the client is no longer subscribed to.
        context_id: Uuid,
    },
    /// A new message was created in a subscribed channel / conversation.
    MessageCreate {
        /// Raw NATS event payload forwarded verbatim.
        data: serde_json::Value,
    },
    /// An existing message was edited.
    MessageUpdate {
        /// Raw NATS event payload forwarded verbatim.
        data: serde_json::Value,
    },
    /// A message was deleted.
    MessageDelete {
        /// Raw NATS event payload forwarded verbatim.
        data: serde_json::Value,
    },
    /// A reaction was added to a message.
    ReactionAdd {
        /// Raw NATS event payload forwarded verbatim.
        data: serde_json::Value,
    },
    /// A reaction was removed from a message.
    ReactionRemove {
        /// Raw NATS event payload forwarded verbatim.
        data: serde_json::Value,
    },
    /// The server encountered an error processing a client message.
    Error {
        /// Human-readable error description.
        message: String,
    },
}
