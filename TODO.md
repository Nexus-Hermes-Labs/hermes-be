# Hermes — TODO

---

## Realtime Service (WebSocket Gateway)

The messaging HTTP API is complete. The missing piece is real-time delivery:
client sends via HTTP → messaging-service publishes to NATS → **realtime-service pushes to WebSocket clients**.

---

### Backend — realtime-service (port 8080)

#### 1. WebSocket endpoint
- `GET /ws?token=<access_token>` — upgrade connection, validate JWT on handshake
- Each connection gets a `ClientSession { user_id, sender: WsSender }`
- Reject if token missing or invalid (close with 4001)

#### 2. Client registry
- `ClientRegistry`: `DashMap<UserId, Vec<WsSender>>` (one user, many tabs/devices)
- Register on connect, remove on disconnect
- Thread-safe, Arc-wrapped in AppState

#### 3. Subscription management
- Client sends JSON opcodes after connecting:
  ```json
  { "op": "SUBSCRIBE",   "channel_id": "uuid" }
  { "op": "UNSUBSCRIBE", "channel_id": "uuid" }
  ```
- `SubscriptionRegistry`: `DashMap<ChannelId, HashSet<UserId>>`
- Covers both guild channels and conversation IDs (same UUID key)

#### 4. NATS → WebSocket bridge
Subscribe to these subjects (published by messaging-service):

| NATS subject | WS op sent to clients |
|---|---|
| `hermes.message.created` | `MESSAGE_CREATE` |
| `hermes.message.updated` | `MESSAGE_UPDATE` |
| `hermes.message.deleted` | `MESSAGE_DELETE` |
| `hermes.reaction.added` | `REACTION_ADD` |
| `hermes.reaction.removed` | `REACTION_REMOVE` |

For each event: parse payload → look up `channel_id` in SubscriptionRegistry → push to all subscribed users via ClientRegistry.

#### 5. Typing indicator (client-initiated)
- Client sends `{ "op": "TYPING_START", "channel_id": "uuid" }` over WS
- realtime-service broadcasts `{ "op": "TYPING_START", "d": { "user_id": "...", "channel_id": "..." } }` to all other subscribers
- No NATS needed — direct fan-out from registry

#### 6. Heartbeat / keepalive
- Server sends `{ "op": "HEARTBEAT" }` every 30s
- Client must respond with `{ "op": "HEARTBEAT_ACK" }` within 10s or get disconnected

#### 7. Traefik routing
Add to `infra/traefik/dynamic/routes.yml`:
```yaml
realtime-ws:
  rule: "PathPrefix(`/ws`)"
  entryPoints: [web]
  middlewares: [rate-limit]   # no jwt-auth — token is in query param
  service: realtime-service
  priority: 50
```

---

### Frontend — wsStore integration

`src/state/wsStore.ts` already exists. Wire it up:

#### 1. Connect on login
- After `setAuthenticated`, call `wsStore.connect(accessToken)`
- URL: `ws://localhost/ws?token=<accessToken>`

#### 2. Disconnect on logout
- Call `wsStore.disconnect()` in logout mutation's `onSuccess`

#### 3. Subscribe to active channel
- On route change to `/channels/:guildId/:channelId` → send `SUBSCRIBE` opcode
- On route change away → send `UNSUBSCRIBE`
- Same for DM conversations (`/channels/@me/:conversationId`)

#### 4. Handle incoming events
In the WS message handler:

| Op | Action |
|---|---|
| `MESSAGE_CREATE` | `messageStore.appendMessage(key, message)` |
| `MESSAGE_UPDATE` | `messageStore.updateMessage(key, message)` |
| `MESSAGE_DELETE` | `messageStore.removeMessage(key, messageId)` |
| `REACTION_ADD` | Invalidate TanStack Query `['messages', id, 'reactions']` |
| `REACTION_REMOVE` | Same |
| `TYPING_START` | Update `uiStore.typingUsers` (auto-clear after 5s) |
| `HEARTBEAT` | Send `HEARTBEAT_ACK` immediately |

#### 5. Auto-reconnect
- On close (non-4001): exponential backoff reconnect (1s → 2s → 4s → max 30s)
- On 4001 (auth failure): call token refresh → reconnect with new token

---

## After Realtime — Next Services

| Service | Purpose | Priority |
|---|---|---|
| `presence-service` (8087) | Online/offline/idle/DND status, sync with WebSocket connect/disconnect | Medium |
| `media-service` (8088) | File/image upload, avatar storage, CDN proxy | Medium |
| `notification-service` (8089) | Unread counts, @mention push notifications | Low |
| `chat-service` (full impl) | Per plan in `.claude/plans/` | Low |
