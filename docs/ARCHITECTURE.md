# Hermes - System Architecture

**Version:** 3.0.0
**Last Updated:** March 25, 2026

## Table of Contents

- [Overview](#overview)
- [Architecture Goals](#architecture-goals)
- [Service Map](#service-map)
- [Communication Architecture](#communication-architecture)
  - [Traefik Edge Proxy](#traefik-edge-proxy)
  - [gRPC Inter-Service Communication](#grpc-inter-service-communication)
  - [NATS Event Bus](#nats-event-bus)
  - [WebSocket Real-Time Layer](#websocket-real-time-layer)
- [Data Architecture](#data-architecture)
- [Authentication and Authorization](#authentication-and-authorization)
- [AI Translation Pipeline](#ai-translation-pipeline)
- [Observability](#observability)
- [Security Architecture](#security-architecture)
- [Technology Stack](#technology-stack)
- [Deployment Architecture](#deployment-architecture)
- [Scalability Strategy](#scalability-strategy)

---

## Overview

Hermes is a microservices-based real-time communication platform built in Rust. The system follows Domain-Driven Design (DDD) principles with a hybrid communication architecture. The key differentiator is AI-powered real-time translation integrated at the infrastructure level.

Traefik serves as the edge proxy handling REST routing, TLS termination, and rate limiting. Backend services focus purely on domain logic. A dedicated `realtime-service` handles WebSocket connections and NATS event fanout.

For the canonical code patterns used inside each service (DDD layers, Repository, Unit of Work, error handling, etc.) see [PATTERNS.md](./PATTERNS.md).

---

## Architecture Goals

| Goal | Target |
|---|---|
| Scalability | Handle millions of concurrent users |
| Low Latency | <100ms p95 text, <200ms AI translation |
| High Availability | 99.9% uptime SLA |
| Maintainability | Clean DDD with strict lint policy |
| Multilingual | Translation as a first-class concern |

---

## Service Map

| Service | Port | Domain | Phase |
|---|---|---|---|
| **Traefik** | 80/443 | REST reverse proxy, TLS, rate limiting, CORS | Infrastructure |
| **auth-service** | 8081 | Registration, login, JWT, sessions | MVP |
| **user-service** | 8082 | Profiles, friends, blocks, privacy | MVP |
| **guild-service** | 8086 | Guilds, roles, members, invites, permissions | MVP |
| **channel-service** | 8083 | Text/voice channels, categories | MVP |
| **chat-service** | 8084 | Messages, reactions, attachments, history | MVP |
| **realtime-service** | 8092 | WebSocket, NATS event fanout | MVP |
| **presence-service** | 8087 | Online/offline/idle/DND, typing indicators | Phase 2 |
| **media-service** | 8088 | File uploads, image processing, CDN | Phase 2 |
| **notification-service** | 8089 | Push notifications, unread counts, mentions | Phase 2 |
| **ai-service** | 8091 | Real-time translation, STT, TTS | Phase 3 |
| **search-service** | 8090 | Full-text search across messages/users/guilds | Phase 4 |
| **voice-service** | 8085 | WebRTC signaling, voice channel management | Phase 4 |

```
+--------------------------------------------------+
|            Client Applications                   |
|     (Web, iOS, Android, Desktop)                 |
+--------+-----------------------+-----------------+
         | REST (HTTPS)          | WebSocket
         v                       v
+------------------+    +------------------+
|     Traefik      |    | realtime-service |
| (reverse proxy)  |    |     (8092)       |
+--+---+---+---+---+    +--------+---------+
   |   |   |   |                 |
   v   v   v   v            +----+----+
 Auth User Guild Chan Chat  |  NATS   |
 8081 8082 8086 8083 8084   +---------+
   |    |    |    |    |
   +----+----+----+----+
              |
     +--------+--------+
     |                 |
+----+-----+    +------+----+
|PostgreSQL |    |  Redis    |
| (Shared)  |    | (Cache)   |
+-----------+    +----------+
```

---

## Communication Architecture

### Traefik Edge Proxy

Traefik is the only entry point for REST traffic. It handles all edge concerns so backend services don't have to.

**Responsibilities:**
- REST routing by URL prefix (`/v1/auth/*` → auth-service, etc.)
- TLS 1.3 termination
- Rate limiting (100 req/s average, 50 burst)
- JWT validation via ForwardAuth middleware (delegates to auth-service)
- CORS headers
- gzip compression
- Health check monitoring

**What Traefik does NOT handle:**
- WebSocket connections (realtime-service owns those)
- gRPC between services (direct, not proxied)
- Event fanout (NATS + realtime-service)

### gRPC Inter-Service Communication

Used for synchronous service-to-service queries that need an immediate response. Goes direct between services on internal network, never through Traefik.

```
auth-service ──gRPC──► user-service     (discriminator generation, username checks)
guild-service ──gRPC──► user-service    (user lookups for member management)
channel-service ──gRPC──► guild-service (permission verification)
```

Proto files live in `proto/`. Code generation runs at build time via `build.rs` using `tonic-build`. Each gRPC client is wrapped in a domain-aware struct that implements a trait (e.g., `UserProfileClient`), keeping Tonic types out of the application layer.

Both HTTP and gRPC servers run concurrently on separate ports (e.g., HTTP :8081, gRPC :50081).

### NATS Event Bus

Used for asynchronous cross-service communication and eventual consistency. Services publish domain events after successful writes; other services subscribe independently.

**Event categories:**

| Subject | Publisher | Subscribers |
|---|---|---|
| `user.created` | auth-service | user-service, notification-service |
| `user.updated` | user-service | realtime-service |
| `guild.created` | guild-service | realtime-service |
| `member.joined` | guild-service | realtime-service, notification-service |
| `message.created` | chat-service | realtime-service, ai-service |
| `translation.completed` | ai-service | realtime-service |

The `EventPublisher` trait in `common` abstracts NATS. The `NatsPublisher` struct implements it.

### WebSocket Real-Time Layer

Clients open a persistent WebSocket connection to `realtime-service`. The service:

1. Validates the JWT on connection upgrade
2. Subscribes to NATS subjects relevant to that user (guilds they're in, DMs, etc.)
3. Fans incoming NATS events out to the connected client over WebSocket
4. Tracks connection state in Redis for horizontal scaling

```
Client ──WS──► realtime-service ──subscribe──► NATS
                     │
              push events to client
```

---

## Data Architecture

### MVP: Shared PostgreSQL

All services share one PostgreSQL database. Migrations live in `services/common/migrations/` and run in timestamp order.

**Rationale:** Simpler for MVP — no distributed transactions, ACID guarantees, easy local development.

**Trade-offs:** Tight coupling, single point of failure, limited per-service scaling. Post-MVP will move to database-per-service.

SQLx with compile-time query checking is used throughout. All queries are verified against the live schema at compile time via `sqlx::query!()`.

### Post-MVP: Database per Service

```
+-------------+  +-------------+  +-------------+  +-------------+
| Auth DB     |  | User DB     |  | Guild DB    |  | Chat DB     |
| - sessions  |  | - profiles  |  | - guilds    |  | - messages  |
| - tokens    |  | - friends   |  | - members   |  | - reactions |
|             |  | - blocks    |  | - roles     |  | - attachments|
+-------------+  +-------------+  +-------------+  +-------------+
```

### Caching Strategy (Redis)

| Cache | TTL | Purpose |
|---|---|---|
| Session cache | 15 min | JWT claims, active sessions |
| User profile | 1 hr | Frequently accessed user data |
| Relationship | 5 min | `are_friends(A,B)`, `is_blocked(A,B)` |
| Presence | live | Online status, last seen |
| Message cache | — | Last 50 messages per channel |
| Connection state | live | Which users are connected to realtime-service |

---

## Authentication and Authorization

### JWT Token Flow

```
1. POST /v1/auth/login → auth-service validates credentials (Argon2)
2. auth-service generates: access token (6h, audience "api")
                           refresh token (30d, audience "auth")
3. Client stores tokens
4. Client sends: Authorization: Bearer <access_token>
5. Traefik ForwardAuth → GET auth-service/internal/verify
6. auth-service validates signature, expiry, audience, type
7. On success: injects X-User-Id, X-User-Role, X-User-Email headers
8. Request forwarded to backend service
9. RequestUser extractor reads headers → typed struct in handler
```

JWT claims include: `sub` (user id), `email`, `role` (SystemRole), `typ` (Access/Refresh), standard RFC 7519 fields.

For the `RequestUser` extractor implementation see [PATTERNS.md — Gateway Authentication Pattern](./PATTERNS.md#10-gateway-authentication-pattern).

### Authorization Levels

- **Resource ownership**: Users can only modify their own data
- **Role-based system-wide**: `SystemRole` — User, Moderator, Admin
- **Guild-level RBAC**: Bitflag permission system with role hierarchy. Guild owner → admin roles → custom roles with granular permission bits (SEND_MESSAGES, KICK_MEMBERS, MANAGE_CHANNELS, etc.)

---

## AI Translation Pipeline

```
User sends message
       │
       ▼
  chat-service (stores original, publishes NATS: message.created)
       │
       ▼
  ai-service (detects language, translates to target languages)
       │
       ▼  NATS: translation.completed
  realtime-service (delivers translated message to recipients
                    based on their language preferences)
```

**Phase 3a — Text**: Language detection + translation, original preserved alongside translations.

**Phase 3b — Voice STT**: Audio stream from voice-service → ai-service → real-time transcription + translated subtitles.

**Phase 3c — Voice TTS**: Transcribed text → synthesized speech in target language. Target: <500ms speech-to-speech end-to-end.

---

## Observability

### Tracing

Structured logging via `tracing` + `tracing-subscriber`. Output is JSON in production, pretty in development. ERROR logs are also written to a file appender. Request IDs are propagated across services via headers.

### Metrics (Prometheus)

```
http_requests_total{method, status, service}
http_request_duration_seconds{method, path, service}
grpc_requests_total{method, status, service}
websocket_connections_active
db_connections_active
cache_hits_total / cache_misses_total
nats_messages_published / nats_messages_consumed
```

### Health Checks

Each service exposes:
- `GET /health/live` — liveness (is the process alive?)
- `GET /health/ready` — readiness (is the DB and Redis reachable?)

---

## Security Architecture

```
+------------------------------------------+
| Layer 1: Network (Firewall, DDoS)        |
+------------------------------------------+
| Layer 2: Traefik (TLS, rate limit, CORS) |
+------------------------------------------+
| Layer 3: Authentication (JWT ForwardAuth)|
+------------------------------------------+
| Layer 4: Authorization (RBAC + bitflags) |
+------------------------------------------+
| Layer 5: Data (Argon2, parameterized SQL)|
+------------------------------------------+
```

**Measures:**
- TLS 1.3 termination at Traefik
- Rate limiting at edge (100 req/s) + Governor crate on auth endpoints as defense-in-depth
- Argon2 password hashing with salt
- JWT HS256 with separate secrets for access and refresh tokens
- All SQL via SQLx parameterized queries (compile-time checked — no injection possible)
- `validator` crate on all request DTOs
- `unsafe_code` forbidden, `unwrap_used`/`panic` denied at workspace level
- Backend services not exposed directly — only accessible from Traefik and each other

---

## Technology Stack

```
Language:        Rust (stable)
Edge Proxy:      Traefik (REST routing, TLS, rate limiting, ForwardAuth)
Web Framework:   Axum 0.7 (async, Tower middleware)
gRPC:            Tonic 0.11 (Protocol Buffers, HTTP/2)
Database:        PostgreSQL 16 (SQLx 0.8, compile-time checked)
Cache:           Redis 7 (sessions, presence, connection state)
Messaging:       NATS (async event bus, JetStream for persistence)
Auth:            jsonwebtoken 9.3, Argon2, SHA256 token hashing
Real-time:       WebSocket (tokio-tungstenite) via realtime-service
API Docs:        utoipa 5 (OpenAPI / Swagger)
Observability:   tracing, tracing-subscriber, Prometheus, Grafana
Testing:         testcontainers, mockall, fake, rstest
```

---

## Deployment Architecture

### Development (Docker Compose)

Infrastructure services run in Docker; Rust services run natively via `cargo`.

```
docker-compose provides:
  - Traefik       (port 80)
  - PostgreSQL 16 (port 5432)
  - Redis 7       (port 6379)
  - NATS          (port 4222, monitoring 8222)
  - Prometheus    (port 9090)
  - Grafana       (port 3000)

Run services:
  make run-auth     # cargo run --bin auth-service
  make run-user     # cargo run --bin user-service
  make dev          # full environment (docker + migrate + seed)
```

### Production (Future — Kubernetes)

- Traefik Ingress Controller at the edge
- Pod autoscaling per service (CPU / request metrics)
- Kubernetes DNS for service discovery
- Rolling updates with health check gates
- Secrets via Kubernetes Secrets / Vault

---

## Scalability Strategy

### Horizontal Scaling

All services are stateless — no in-memory session state. Any pod can handle any request. Independent scaling per service.

- Auth state: JWT (stateless) + Redis sessions
- Guild/channel context: fetched from DB or Redis per request
- realtime-service scales horizontally with Redis-backed connection state

### Database Scaling Path

1. **MVP**: Shared PostgreSQL, SQLx connection pool
2. **Post-MVP**: Database per service, read replicas
3. **Scale**: PgBouncer, sharding by `guild_id` or `user_id`

### Performance Targets

| Metric | Target |
|---|---|
| p50 response time | <50ms |
| p95 response time | <100ms |
| p99 response time | <200ms |
| AI translation (text) | <200ms |
| AI translation (voice) | <500ms end-to-end |
| Throughput per pod | 10k req/s |
