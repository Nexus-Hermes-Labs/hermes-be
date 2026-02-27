//! WebSocket upgrade handler and per-connection task management.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::presentation::ws::messages::{ClientMsg, ServerMsg};
use crate::state::AppState;

/// Query parameters expected on the WebSocket upgrade request.
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT access token supplied by the client.
    pub token: String,
}

/// Axum handler for `GET /ws?token=<jwt>`.
///
/// Validates the supplied JWT, then upgrades the connection to a WebSocket.
/// Returns `401 Unauthorized` when the token is missing or invalid.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let claims = match state.jwt_manager.verify_access_token(&params.token) {
        Ok(c) => c,
        Err(e) => {
            warn!("WS auth failed: {e}");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let user_id = match claims.user_id() {
        Ok(id) => id,
        Err(e) => {
            warn!("WS claims.sub is not a valid UUID: {e}");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

// ── Per-connection logic ──────────────────────────────────────────────────────

async fn handle_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Register this connection.
    state.client_registry.insert(user_id, tx);

    // Send HELLO immediately after upgrade.
    let hello = ServerMsg::Hello {
        heartbeat_interval: 30_000,
    };
    if let Ok(json) = serde_json::to_string(&hello) {
        let _ = ws_tx.send(Message::Text(json)).await;
    }

    // Spawn a writer task: forwards queued ServerMsg JSON to the WS stream.
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Read loop: handle ClientMsg frames from the WebSocket.
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(m) => m,
            Err(e) => {
                debug!("WS read error for user {user_id}: {e}");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                process_client_msg(&text, user_id, &state);
            }
            Message::Close(_) => break,
            // Ignore Ping / Pong / Binary frames.
            _ => {}
        }
    }

    // Cleanup on disconnect.
    write_task.abort();
    state.client_registry.remove(&user_id);
    remove_all_subscriptions(user_id, &state);

    debug!("WS connection closed for user {user_id}");
}

// ── Client message processing ─────────────────────────────────────────────────

fn process_client_msg(raw: &str, user_id: Uuid, state: &AppState) {
    let client_msg: ClientMsg = match serde_json::from_str(raw) {
        Ok(m) => m,
        Err(_) => {
            send_error(user_id, "Invalid message format", state);
            return;
        }
    };

    match client_msg {
        ClientMsg::Subscribe { context_id } => {
            state
                .sub_registry
                .entry(context_id)
                .or_default()
                .insert(user_id);

            let ack = ServerMsg::Subscribed { context_id };
            send_msg(user_id, &ack, state);
            debug!("User {user_id} subscribed to {context_id}");
        }

        ClientMsg::Unsubscribe { context_id } => {
            if let Some(users) = state.sub_registry.get(&context_id) {
                users.remove(&user_id);
            }
            let ack = ServerMsg::Unsubscribed { context_id };
            send_msg(user_id, &ack, state);
            debug!("User {user_id} unsubscribed from {context_id}");
        }

        ClientMsg::Heartbeat => {
            send_msg(user_id, &ServerMsg::HeartbeatAck, state);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn send_msg(user_id: Uuid, msg: &ServerMsg, state: &AppState) {
    if let Ok(json) = serde_json::to_string(msg) {
        if let Some(sender) = state.client_registry.get(&user_id) {
            let _ = sender.send(json);
        }
    }
}

fn send_error(user_id: Uuid, message: &str, state: &AppState) {
    let err = ServerMsg::Error {
        message: message.to_string(),
    };
    send_msg(user_id, &err, state);
}

fn remove_all_subscriptions(user_id: Uuid, state: &AppState) {
    for entry in state.sub_registry.iter() {
        entry.value().remove(&user_id);
    }
}
