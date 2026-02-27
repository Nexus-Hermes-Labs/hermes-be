//! NATS wildcard subscriber that fans out messaging events to connected WebSocket
//! clients.
//!
//! Subjects consumed (published by `messaging-service`):
//! - `hermes.message.created`  — `{message_id, channel_id?, conversation_id?, user_id, content, created_at}`
//! - `hermes.message.updated`  — `{message_id, channel_id?, conversation_id?, content, edited_at}`
//! - `hermes.message.deleted`  — `{message_id, channel_id?, conversation_id?, user_id}`
//! - `hermes.reaction.added`   — `{message_id, user_id, emoji}` (no context_id; resolved via cache)
//! - `hermes.reaction.removed` — `{message_id, user_id, emoji}` (no context_id; resolved via cache)

use futures::StreamExt;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::presentation::ws::messages::ServerMsg;
use crate::state::{AppState, ClientRegistry, MsgContextCache, SubscriptionRegistry};

const SUBJECT_WILDCARD: &str = "hermes.>";

/// Long-running task: subscribes to NATS and fans events out to WS clients.
///
/// This function never returns under normal operation; it exits only if the
/// NATS subscription itself is dropped (e.g. server disconnect).
pub async fn run(state: AppState) {
    let mut subscriber = match state.nats.subscribe(SUBJECT_WILDCARD).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to subscribe to NATS subject '{SUBJECT_WILDCARD}': {e}");
            return;
        }
    };

    debug!("NATS listener active on '{SUBJECT_WILDCARD}'");

    while let Some(msg) = subscriber.next().await {
        let subject = msg.subject.as_str().to_owned();
        let payload: serde_json::Value = match serde_json::from_slice(&msg.payload) {
            Ok(v) => v,
            Err(e) => {
                warn!("NATS parse error on '{subject}': {e}");
                continue;
            }
        };

        process_event(
            &subject,
            payload,
            &state.client_registry,
            &state.sub_registry,
            &state.msg_ctx_cache,
        );
    }

    warn!("NATS subscription '{SUBJECT_WILDCARD}' ended — listener task exiting");
}

// ── Event dispatching ─────────────────────────────────────────────────────────

fn process_event(
    subject: &str,
    payload: serde_json::Value,
    clients: &ClientRegistry,
    subs: &SubscriptionRegistry,
    cache: &MsgContextCache,
) {
    match subject {
        "hermes.message.created" => {
            // Cache message_id → context_id for later reaction routing.
            if let Some(context_id) = extract_context_id(&payload) {
                if let Some(message_id) = extract_uuid(&payload, "message_id") {
                    cache.insert(message_id, context_id);
                }
                let server_msg = ServerMsg::MessageCreate { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "hermes.message.updated" => {
            if let Some(context_id) = extract_context_id(&payload) {
                let server_msg = ServerMsg::MessageUpdate { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "hermes.message.deleted" => {
            if let Some(context_id) = extract_context_id(&payload) {
                let server_msg = ServerMsg::MessageDelete { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "hermes.reaction.added" => {
            if let Some(context_id) = resolve_reaction_context(&payload, cache) {
                let server_msg = ServerMsg::ReactionAdd { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "hermes.reaction.removed" => {
            if let Some(context_id) = resolve_reaction_context(&payload, cache) {
                let server_msg = ServerMsg::ReactionRemove { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        // Other subjects (conversation events, etc.) are ignored for now.
        _ => {}
    }
}

// ── Fan-out ───────────────────────────────────────────────────────────────────

/// Serialise `msg` and deliver it to every user subscribed to `context_id`.
fn fan_out(
    context_id: Uuid,
    msg: &ServerMsg,
    clients: &ClientRegistry,
    subs: &SubscriptionRegistry,
) {
    let json = match serde_json::to_string(msg) {
        Ok(j) => j,
        Err(e) => {
            warn!("Failed to serialise ServerMsg: {e}");
            return;
        }
    };

    let Some(subscribers) = subs.get(&context_id) else {
        return; // No one is listening to this context.
    };

    for user_id in subscribers.iter() {
        let Some(sender) = clients.get(&*user_id) else {
            continue; // Client disconnected between checks; harmless.
        };

        if sender.send(json.clone()).is_err() {
            // The receiver was dropped — the WS connection handler will
            // clean up the registry entry on its next iteration.
            debug!("WS sender for user {} is closed; skipping", *user_id);
        }
    }
}

// ── Payload helpers ───────────────────────────────────────────────────────────

/// Extract a UUID field from a JSON object.
fn extract_uuid(payload: &serde_json::Value, field: &str) -> Option<Uuid> {
    payload
        .get(field)
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Extract the `context_id` from a message payload.
///
/// Messages target exactly one of `channel_id` or `conversation_id`; the other
/// will be `null` or absent.
fn extract_context_id(payload: &serde_json::Value) -> Option<Uuid> {
    extract_uuid(payload, "channel_id").or_else(|| extract_uuid(payload, "conversation_id"))
}

/// Look up the `context_id` for a reaction event from the in-memory cache.
///
/// Reactions carry only `message_id`; the cache was populated when the
/// corresponding `hermes.message.created` event was processed.
fn resolve_reaction_context(payload: &serde_json::Value, cache: &MsgContextCache) -> Option<Uuid> {
    let message_id = extract_uuid(payload, "message_id")?;
    cache.get(&message_id).map(|v| *v)
}
