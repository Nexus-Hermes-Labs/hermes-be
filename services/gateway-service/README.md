# Gateway Service

WebSocket gateway and REST API entry point for Hermes.

## Responsibilities

- WebSocket connection management
- Real-time event broadcasting
- REST API gateway
- Request routing to microservices
- Connection authentication
- Rate limiting
- Load balancing

## WebSocket Events

### Client → Server Events

```typescript
// Connect
{
  "op": 0,
  "d": {
    "token": "jwt_token"
  }
}

// Heartbeat
{
  "op": 1,
  "d": null
}

// Send Message
{
  "op": 2,
  "t": "MESSAGE_CREATE",
  "d": {
    "channel_id": "uuid",
    "content": "Hello, world!"
  }
}

// Voice State Update
{
  "op": 4,
  "t": "VOICE_STATE_UPDATE",
  "d": {
    "channel_id": "uuid",
    "self_mute": false,
    "self_deaf": false
  }
}
```

### Server → Client Events

```typescript
// Ready Event (on connection)
{
  "op": 0,
  "t": "READY",
  "d": {
    "user": { /* user_profile object */ },
    "servers": [ /* server objects */ ],
    "session_id": "uuid"
  }
}

// Message Create
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "d": {
    "id": "uuid",
    "channel_id": "uuid",
    "author": { /* user_profile object */ },
    "content": "Hello!",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}

// Presence Update
{
  "op": 0,
  "t": "PRESENCE_UPDATE",
  "d": {
    "user_id": "uuid",
    "status": "online",
    "activities": []
  }
}

// Voice State Update
{
  "op": 0,
  "t": "VOICE_STATE_UPDATE",
  "d": {
    "channel_id": "uuid",
    "user_id": "uuid",
    "session_id": "uuid",
    "self_mute": false,
    "self_deaf": false
  }
}

// Heartbeat ACK
{
  "op": 11,
  "d": null
}
```

## REST API Endpoints

The gateway proxies REST API requests to appropriate microservices:

- `/api/auth/*` → Auth Service
- `/api/users/*` → User Service
- `/api/servers/*` → Channel Service
- `/api/channels/*` → Channel Service / Chat Service
- `/api/voice/*` → Voice Service
- `/api/stream/*` → Stream Service

## Environment Variables

```bash
NATS_URL=nats://localhost:4222
REDIS_URL=redis://:password@localhost:6379
JWT_SECRET=your-secret-key
PORT=8080
WS_PORT=8080
AUTH_SERVICE_URL=http://localhost:8081
USER_SERVICE_URL=http://localhost:8082
CHANNEL_SERVICE_URL=http://localhost:8083
CHAT_SERVICE_URL=http://localhost:8084
VOICE_SERVICE_URL=http://localhost:8085
STREAM_SERVICE_URL=http://localhost:8086
```

## Running

```bash
cargo run --bin gateway-service
```

WebSocket: ws://localhost:8080
REST API: http://localhost:8080/api
