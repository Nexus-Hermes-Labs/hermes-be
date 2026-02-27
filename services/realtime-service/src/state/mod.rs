//! Application state shared across the HTTP/WS layer and the NATS listener.

use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::Metrics;
use dashmap::{DashMap, DashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Sends serialised `ServerMsg` JSON strings to a connected WebSocket client.
pub type WsSender = mpsc::UnboundedSender<String>;

/// Maps `user_id` → WS sender for a connected client.
pub type ClientRegistry = DashMap<Uuid, WsSender>;

/// Maps a `context_id` (channel or conversation UUID) → set of subscribed
/// `user_id`s that want to receive events for that context.
pub type SubscriptionRegistry = DashMap<Uuid, DashSet<Uuid>>;

/// Caches `message_id` → `context_id` so that reaction events (which carry
/// only a `message_id`) can be routed to the correct subscribers.
pub type MsgContextCache = DashMap<Uuid, Uuid>;

/// Shared application state injected into every HTTP/WS handler and the NATS
/// listener task.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AppState {
    /// Live WebSocket connections indexed by the authenticated user's UUID.
    pub client_registry: Arc<ClientRegistry>,
    /// Channel / conversation subscriptions per connected user.
    pub sub_registry: Arc<SubscriptionRegistry>,
    /// Ephemeral cache mapping `message_id` → `context_id` (channel or conv).
    pub msg_ctx_cache: Arc<MsgContextCache>,
    /// NATS async client (used by the listener task).
    pub nats: async_nats::Client,
    /// JWT manager used to validate tokens supplied in the `?token=` query
    /// parameter during WebSocket upgrade.
    pub jwt_manager: Arc<JwtManager>,
    /// Prometheus metrics handle.
    pub metrics: Metrics,
}
