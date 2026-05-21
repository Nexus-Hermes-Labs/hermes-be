//! JetStream listener that fans messaging events out to connected WebSocket
//! clients. One ephemeral push consumer is attached per upstream stream;
//! crashes lose events the same way the WebSocket itself does, so durability
//! is intentionally not maintained here.
//!
//! Streams consumed:
//! - `CHAT_EVENTS` — `chat.message.{created,updated,deleted}`,
//!   `chat.reaction.{added,removed}`
//! - `MESSAGING_EVENTS` — `messaging.message.{created,updated,deleted}`,
//!   `messaging.reaction.{added,removed}`
//!
//! Payloads arrive as `EventEnvelope`s produced by the outbox publishers; the
//! domain payload lives under the envelope's `payload` field.

use std::time::Duration;

use async_nats::jetstream::{self, consumer::DeliverPolicy};
use futures::StreamExt;
use tracing::{debug, error, warn};
use uuid::Uuid;

use common::infrastructure::outbox::{ensure_stream, OutboxStreamConfig};

use crate::presentation::ws::messages::ServerMsg;
use crate::state::{AppState, ClientRegistry, MsgContextCache, SubscriptionRegistry};

const CONSUMER_INACTIVE_TIMEOUT: Duration = Duration::from_secs(60);

/// Long-running task: attaches ephemeral consumers to every upstream stream
/// and dispatches events to connected WebSocket clients. Returns only when
/// every per-stream listener has exited.
pub async fn run(state: AppState) {
    let jetstream = jetstream::new(state.nats.clone());

    let chat_config = OutboxStreamConfig::new("CHAT_EVENTS", vec!["chat.>".to_string()]);
    let messaging_config =
        OutboxStreamConfig::new("MESSAGING_EVENTS", vec!["messaging.>".to_string()]);

    let chat = listen(jetstream.clone(), chat_config, "chat.>".to_string(), state.clone());
    let messaging = listen(
        jetstream,
        messaging_config,
        "messaging.>".to_string(),
        state,
    );

    tokio::join!(chat, messaging);
}

async fn listen(
    jetstream: jetstream::Context,
    stream_config: OutboxStreamConfig,
    filter_subject: String,
    state: AppState,
) {
    let stream = match ensure_stream(&jetstream, &stream_config).await {
        Ok(s) => s,
        Err(e) => {
            error!(stream = %stream_config.name, error = %e, "Failed to ensure JetStream stream");
            return;
        }
    };

    let consumer = match stream
        .create_consumer(jetstream::consumer::pull::Config {
            deliver_policy: DeliverPolicy::New,
            filter_subject: filter_subject.clone(),
            inactive_threshold: CONSUMER_INACTIVE_TIMEOUT,
            ..Default::default()
        })
        .await
    {
        Ok(c) => c,
        Err(e) => {
            error!(stream = %stream_config.name, error = %e, "Failed to create ephemeral consumer");
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            error!(stream = %stream_config.name, error = %e, "Failed to open consumer message stream");
            return;
        }
    };

    debug!(stream = %stream_config.name, "JetStream listener active");

    while let Some(message) = messages.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "Failed to receive JetStream message");
                continue;
            }
        };
        let subject = message.subject.as_str().to_owned();

        let envelope: serde_json::Value = match serde_json::from_slice(&message.payload) {
            Ok(v) => v,
            Err(e) => {
                warn!(subject = %subject, error = %e, "JetStream payload parse error");
                let _ = message.ack().await;
                continue;
            }
        };

        let domain_payload = envelope
            .get("payload")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        process_event(
            &subject,
            domain_payload,
            &state.client_registry,
            &state.sub_registry,
            &state.msg_ctx_cache,
        );

        if let Err(e) = message.ack().await {
            warn!(subject = %subject, error = %e, "Failed to ack JetStream message");
        }
    }

    warn!(stream = %stream_config.name, "JetStream consumer message stream ended");
}

// ── Event dispatching ─────────────────────────────────────────────────────────

fn process_event(
    subject: &str,
    payload: serde_json::Value,
    clients: &ClientRegistry,
    subs: &SubscriptionRegistry,
    cache: &MsgContextCache,
) {
    // Subject suffix carries the semantic action regardless of which service
    // produced the event (`chat.message.created`, `messaging.message.created`).
    let suffix = subject.split_once('.').map(|(_, rest)| rest).unwrap_or(subject);

    match suffix {
        "message.created" => {
            if let Some(context_id) = extract_context_id(&payload) {
                if let Some(message_id) = extract_uuid(&payload, "message_id") {
                    cache.insert(message_id, context_id);
                }
                let server_msg = ServerMsg::MessageCreate { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "message.updated" => {
            if let Some(context_id) = extract_context_id(&payload) {
                let server_msg = ServerMsg::MessageUpdate { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "message.deleted" => {
            if let Some(context_id) = extract_context_id(&payload) {
                let server_msg = ServerMsg::MessageDelete { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "reaction.added" => {
            if let Some(context_id) = resolve_reaction_context(&payload, cache) {
                let server_msg = ServerMsg::ReactionAdd { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        "reaction.removed" => {
            if let Some(context_id) = resolve_reaction_context(&payload, cache) {
                let server_msg = ServerMsg::ReactionRemove { data: payload };
                fan_out(context_id, &server_msg, clients, subs);
            }
        }

        _ => {}
    }
}

// ── Fan-out ───────────────────────────────────────────────────────────────────

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
        return;
    };

    for user_id in subscribers.iter() {
        let Some(sender) = clients.get(&*user_id) else {
            continue;
        };

        if sender.send(json.clone()).is_err() {
            debug!("WS sender for user {} is closed; skipping", *user_id);
        }
    }
}

// ── Payload helpers ───────────────────────────────────────────────────────────

fn extract_uuid(payload: &serde_json::Value, field: &str) -> Option<Uuid> {
    payload
        .get(field)
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}

fn extract_context_id(payload: &serde_json::Value) -> Option<Uuid> {
    extract_uuid(payload, "channel_id").or_else(|| extract_uuid(payload, "conversation_id"))
}

fn resolve_reaction_context(payload: &serde_json::Value, cache: &MsgContextCache) -> Option<Uuid> {
    let message_id = extract_uuid(payload, "message_id")?;
    cache.get(&message_id).map(|v| *v)
}
