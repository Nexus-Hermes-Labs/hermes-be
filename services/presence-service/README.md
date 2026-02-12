# Presence Service

Real-time user presence and status management service for Hermes.

## Responsibilities

- Track online/offline/away/dnd status
- Custom status messages
- Activity tracking (Playing, Streaming, etc.)
- Typing indicators
- Last seen timestamps
- Real-time presence broadcasting

## Status Types

- `online` - User is online and active
- `idle` - User is away/idle
- `dnd` - Do not disturb
- `offline` - User is offline or invisible

## API Endpoints

### Update Status
```http
POST /presence/status
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "online",
  "custom_status": "Working on a project",
  "activity": {
    "type": "playing",
    "name": "Rust Programming"
  }
}
```

### Get User Presence
```http
GET /presence/{user_id}
Authorization: Bearer <token>
```

Response:
```json
{
  "user_id": "uuid",
  "status": "online",
  "custom_status": "Working on a project",
  "activity": {
    "type": "playing",
    "name": "Rust Programming",
    "started_at": "2024-01-01T12:00:00Z"
  },
  "last_seen": "2024-01-01T12:30:00Z"
}
```

### Bulk Get Presence
```http
POST /presence/bulk
Authorization: Bearer <token>
Content-Type: application/json

{
  "user_ids": ["uuid1", "uuid2", "uuid3"]
}
```

### Typing Indicator
```http
POST /presence/typing
Authorization: Bearer <token>
Content-Type: application/json

{
  "channel_id": "uuid"
}
```

## Redis Data Structure

### User Presence
```
Key: presence:user:{user_id}
Type: Hash
Fields:
  - status: "online" | "idle" | "dnd" | "offline"
  - custom_status: String
  - activity_type: "playing" | "streaming" | "listening"
  - activity_name: String
  - last_seen: Unix timestamp
TTL: 5 minutes (refreshed on heartbeat)
```

### Typing Indicators
```
Key: typing:{channel_id}
Type: Sorted Set
Score: Unix timestamp
Member: user_id
TTL: 10 seconds
```

## WebSocket Events (via Gateway)

### Status Update
```typescript
{
  "op": 3,
  "t": "PRESENCE_UPDATE",
  "d": {
    "user_id": "uuid",
    "status": "online",
    "custom_status": "Working",
    "activity": { /* activity object */ }
  }
}
```

### Typing Start
```typescript
{
  "op": 8,
  "t": "TYPING_START",
  "d": {
    "channel_id": "uuid",
    "user_id": "uuid",
    "timestamp": 1234567890
  }
}
```

## NATS Events

### Published Events
- `presence.status.changed` - User status changed
- `presence.typing.started` - User started typing
- `presence.activity.updated` - User activity changed

### Subscribed Events
- `gateway.connection.opened` - Set user online
- `gateway.connection.closed` - Set user offline
- `gateway.heartbeat.received` - Refresh presence TTL
- `voice.session.created` - Update activity
- `voice.session.ended` - Clear voice activity

## Heartbeat System

Clients must send heartbeats every 30 seconds to maintain online status:

```typescript
// Client sends via WebSocket
{
  "op": 1,
  "d": null
}

// Server responds
{
  "op": 11,
  "d": null
}
```

If no heartbeat is received for 45 seconds, the user is marked as offline.

## Environment Variables

```bash
REDIS_URL=redis://:password@localhost:6379
NATS_URL=nats://localhost:4222
PORT=8087
HEARTBEAT_INTERVAL=30
HEARTBEAT_TIMEOUT=45
TYPING_TIMEOUT=10
```

## Running

```bash
cargo run --bin presence-service
```

Server starts on: http://localhost:8087
