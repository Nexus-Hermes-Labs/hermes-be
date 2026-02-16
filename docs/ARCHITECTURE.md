# Hermes - System Architecture

**Version:** 2.1.0
**Last Updated:** February 16, 2026

## Table of Contents

- [Overview](#overview)
- [Architecture Principles](#architecture-principles)
- [Service Architecture](#service-architecture)
- [nginx Edge Proxy](#nginx-edge-proxy)
- [Communication Patterns](#communication-patterns)
- [AI Translation Pipeline](#ai-translation-pipeline)
- [Data Architecture](#data-architecture)
- [Authentication and Authorization](#authentication-and-authorization)
- [Technology Stack](#technology-stack)
- [Deployment Architecture](#deployment-architecture)
- [Scalability Strategy](#scalability-strategy)
- [Monitoring and Observability](#monitoring-and-observability)
- [Security Architecture](#security-architecture)

---

## Overview

Hermes is a microservices-based real-time communication platform built with Rust. The system follows Domain-Driven Design (DDD) principles with a hybrid communication architecture. The key differentiator is AI-powered real-time translation integrated at the infrastructure level.

nginx serves as the edge proxy handling REST routing, TLS termination, and rate limiting. Backend services focus purely on domain logic. A dedicated realtime-service handles WebSocket connections and NATS event fanout.

### Architecture Goals

1. **Scalability**: Handle millions of concurrent users
2. **Low Latency**: <100ms p95 for text operations, <200ms for AI translation
3. **High Availability**: 99.9% uptime SLA
4. **Maintainability**: Clean separation of concerns via DDD
5. **Multilingual by Default**: Translation as a first-class infrastructure concern

---

## Architecture Principles

### 1. Microservices Architecture

Each service owns its domain and can be deployed independently. nginx sits at the edge and routes REST traffic.

```
                  nginx (edge proxy)
                        |
    +--------+----------+----------+---------+
    |        |          |          |         |
    v        v          v          v         v
  auth    user       guild     channel    chat     ...
  8081    8082       8086      8083       8084

          realtime-service (8080)
            WebSocket + NATS fanout
```

### 2. Domain-Driven Design (DDD)

Each service follows DDD layers:

```
+--------------------------------------+
|      Presentation Layer              |  <- HTTP/gRPC handlers
+--------------------------------------+
|      Application Layer               |  <- Use cases, DTOs
+--------------------------------------+
|      Domain Layer                    |  <- Business logic, entities
+--------------------------------------+
|      Infrastructure Layer            |  <- Database, gRPC, cache
+--------------------------------------+
```

**Layer Responsibilities:**

- **Domain**: Pure business logic, no dependencies on infrastructure
- **Application**: Orchestrates domain logic, handles transactions
- **Infrastructure**: Implements repositories, external services
- **Presentation**: Handles protocol concerns (HTTP, gRPC)

Additional per-service directories:
- **State**: Shared application state (DB pools, clients)
- **Bootstrap**: Service initialization and dependency wiring

### 3. Hybrid Communication

```
+---------------------------------------------+
| EDGE PROXY (nginx)                          |
| - REST routing to backend services          |
| - TLS termination, rate limiting, CORS      |
+---------------------------------------------+
| SYNCHRONOUS (gRPC)                          |
| - Service-to-service queries                |
| - Operations requiring immediate response   |
| - Strong consistency                        |
+---------------------------------------------+
| ASYNCHRONOUS (NATS)                         |
| - Event notifications                       |
| - AI translation pipeline                   |
| - Cross-service updates                     |
| - Eventual consistency                      |
+---------------------------------------------+
| REAL-TIME (WebSocket via realtime-service)   |
| - Client event streaming                    |
| - NATS event fanout to connected clients    |
+---------------------------------------------+
```

---

## Service Architecture

### 12-Service Overview

| Service | Port | Domain | Phase |
|---------|------|--------|-------|
| **nginx** | 80/443 | REST reverse proxy, TLS, rate limiting, CORS | Infrastructure |
| **auth-service** | 8081 | User registration, login, JWT management, sessions | MVP |
| **user-service** | 8082 | User profiles, friend system, blocks, privacy settings | MVP |
| **guild-service** | 8086 | Guilds, roles, members, invites, permission management | MVP |
| **channel-service** | 8083 | Text/voice channels, categories, channel permissions | MVP |
| **chat-service** | 8084 | Messages, reactions, attachments, message history | MVP |
| **realtime-service** | 8080 | WebSocket connections, NATS event fanout, connection state | MVP |
| **presence-service** | 8087 | Online/offline/idle/DND status, typing indicators | Phase 2 |
| **media-service** | 8088 | File uploads, image processing, CDN proxy, avatars | Phase 2 |
| **notification-service** | 8089 | Push notifications, unread counts, @mentions | Phase 2 |
| **ai-service** | 8091 | Real-time text translation, STT, TTS | Phase 3 |
| **search-service** | 8090 | Full-text search across messages, users, guilds | Phase 4 |
| **voice-service** | 8085 | WebRTC P2P signaling, voice channel management | Phase 4 |

### MVP Architecture

```
+--------------------------------------------------+
|            Client Applications                   |
|     (Web, iOS, Android, Desktop)                |
+--------+-----------------------+-----------------+
         | REST (HTTPS)          | WebSocket
         v                       v
+------------------+    +------------------+
|     nginx        |    | realtime-service |
| (reverse proxy)  |    |     (8080)       |
+--+---+---+---+---+    +--------+---------+
   |   |   |   |                 |
   v   v   v   v            +----+----+
 Auth User Guild Chan Chat  |  NATS   |
 8081 8082 8086  8083 8084  +---------+
   |    |    |    |    |
   +----+----+----+----+
              |
     +--------+--------+
     |                 |
+----+-----+    +-----+----+
|PostgreSQL |    |  Redis   |
| (Shared)  |    | (Cache)  |
+----------+    +----------+
```

### Full Architecture (All Phases)

```
+--------------------------------------------------+
|                  Clients                          |
+----------+-----------------------+---------------+
           | REST                  | WebSocket
           v                       v
    +------+------+        +-------+--------+
    |    nginx    |        |   Realtime     |
    | (80 / 443) |        |    (8080)      |
    +------+------+        +-------+--------+
           |                       |
  +--------+--------+         +----+----+
  |   |   |   |    |         |  NATS   |
  v   v   v   v    v         +----+----+
Auth User Guild Chan Chat         |
8081 8082 8086 8083 8084   +------+------+------+
  |    |    |    |    |    |      |      |      |
  +----+----+----+----+   v      v      v      v
           |             Pres  Media  Notif   AI
     +-----+-----+      8087  8088   8089   8091
     | PostgreSQL |
     +-----------+        +--------+--------+
           |              |        |
     +-----+-----+       v        v
     |   Redis   |     Search   Voice
     +-----------+     8090     8085
```

---

## nginx Edge Proxy

nginx replaces a custom API gateway service, handling all edge concerns:

### Responsibilities

- **REST Routing**: Routes `/v1/auth/*` to auth-service, `/v1/users/*` to user-service, etc.
- **TLS Termination**: HTTPS at the edge, HTTP internally between nginx and services
- **Rate Limiting**: `limit_req` zones per endpoint category (auth endpoints stricter)
- **CORS**: Centralized cross-origin configuration
- **Compression**: gzip for JSON responses
- **Static Files**: Serve OpenAPI/Swagger docs
- **Health Checks**: Upstream health monitoring

### What nginx Does NOT Handle

- **WebSocket connections**: Clients connect directly to realtime-service (nginx can optionally proxy the upgrade, but realtime-service owns the connection lifecycle)
- **Event fanout**: NATS subscription and per-client event delivery is realtime-service's job
- **Authentication logic**: JWT validation happens in each backend service
- **gRPC routing**: Service-to-service gRPC calls go direct, not through nginx

### Example Routing Configuration

```nginx
upstream auth_service {
    server 127.0.0.1:8081;
}
upstream user_service {
    server 127.0.0.1:8082;
}
upstream guild_service {
    server 127.0.0.1:8086;
}
upstream channel_service {
    server 127.0.0.1:8083;
}
upstream chat_service {
    server 127.0.0.1:8084;
}
upstream realtime_service {
    server 127.0.0.1:8080;
}

server {
    listen 80;

    location /v1/auth/ {
        proxy_pass http://auth_service;
    }
    location /v1/users/ {
        proxy_pass http://user_service;
    }
    location /v1/guilds/ {
        proxy_pass http://guild_service;
    }
    location /v1/channels/ {
        proxy_pass http://channel_service;
    }
    location /v1/messages/ {
        proxy_pass http://chat_service;
    }
    location /ws {
        proxy_pass http://realtime_service;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

---

## Communication Patterns

### 1. Client to Backend (via nginx)

All REST API calls from clients go through nginx. nginx routes based on URL prefix to the correct backend service. Each service validates the JWT independently.

```
Client --HTTPS--> nginx --HTTP--> auth-service (8081)
                       |--------> user-service (8082)
                       |--------> guild-service (8086)
                       |--------> channel-service (8083)
                       +--------> chat-service (8084)
```

### 2. Client to Realtime (WebSocket)

Clients establish a WebSocket connection to realtime-service. The service authenticates the connection (JWT on upgrade), subscribes to relevant NATS subjects, and pushes events to the client.

```
Client --WebSocket--> realtime-service (8080) --subscribe--> NATS
                             |
                      push events to client
```

### 3. Synchronous Communication (gRPC)

Used for service-to-service queries that require an immediate response. These go direct between services, not through nginx.

**Current gRPC flows:**
- Auth Service -> User Service: discriminator generation, username availability checks
- Guild Service -> User Service: user lookups for member management
- Channel Service -> Guild Service: permission verification

**Proto definitions** live in `proto/` and are compiled at build time via `build.rs` in each service.

### 4. Asynchronous Communication (NATS)

Used for event-driven workflows and the AI translation pipeline.

**Event categories:**
- **User events**: `user.created`, `user.updated`, `user.deleted`
- **Relationship events**: `friend.request.sent`, `friend.request.accepted`
- **Guild events**: `guild.created`, `member.joined`, `member.left`
- **Message events**: `message.created`, `message.updated`, `message.deleted`
- **AI events**: `translation.requested`, `translation.completed`

realtime-service subscribes to events relevant to each connected client and fans them out over WebSocket.

---

## AI Translation Pipeline

The AI service integrates into the message flow via NATS:

```
User sends message
       |
       v
  chat-service
  (stores original)
       |
       | NATS: message.created
       v
  ai-service
  (detects language, translates)
       |
       | NATS: translation.completed
       v
  realtime-service
  (delivers translated message
   to recipients based on their
   language preferences)
```

### Translation Phases

**Phase 3a -- Text Translation:**
- Language detection on incoming messages
- Translation to target languages based on guild/user preferences
- Original message preserved, translations stored alongside

**Phase 3b -- Voice STT (Speech-to-Text):**
- Audio stream from voice-service piped to ai-service
- Real-time transcription with language detection
- Translated subtitles pushed to clients via realtime-service

**Phase 3c -- Voice TTS (Text-to-Speech):**
- Transcribed and translated text synthesized to speech
- Audio output streamed to target users in their language
- Latency target: <500ms end-to-end for speech-to-speech

---

## Data Architecture

### MVP: Shared Database

```
+--------------------------------------+
|          PostgreSQL 16               |
+--------------------------------------+
|                                      |
|  +------------+    +--------------+  |
|  |   users    |    | user_        |  |
|  |            |    | relationships|  |
|  +------------+    +--------------+  |
|                                      |
|  +------------+    +--------------+  |
|  |   guilds   |    |  channels    |  |
|  +------------+    +--------------+  |
|                                      |
|  +------------+    +--------------+  |
|  |  messages  |    | translations |  |
|  +------------+    +--------------+  |
|                                      |
|  All services share one DB (MVP)    |
+--------------------------------------+
```

**Rationale:** Simple for MVP, no distributed transactions, ACID guarantees, easy local development.

**Trade-offs:** Tight coupling, single point of failure, scaling limitations. Post-MVP will migrate to database-per-service.

### Post-MVP: Database per Service

```
+-------------+  +-------------+  +-------------+  +-------------+
| Auth DB     |  | User DB     |  | Guild DB    |  | Chat DB     |
| - sessions  |  | - users     |  | - guilds    |  | - messages  |
| - tokens    |  | - friends   |  | - members   |  | - reactions |
|             |  | - blocks    |  | - roles     |  | - attaches  |
+-------------+  +-------------+  +-------------+  +-------------+
```

### Caching Strategy

Redis is used for:
- **Session cache** (15min TTL): JWT claims, active sessions
- **User profile cache** (1hr TTL): Frequently accessed user data
- **Relationship cache** (5min TTL): `are_friends(A, B)`, `is_blocked(A, B)` lookups
- **Presence data**: Online status, last seen timestamps
- **Message cache**: Last 50 messages per channel for fast retrieval
- **Connection state**: realtime-service tracks connected users in Redis

### Database Design Principles

- **Bidirectional relationship sync**: PostgreSQL triggers maintain consistency for friend/block relationships
- **Covering indexes**: Optimized for common query patterns (friends list, pending requests)
- **Compile-time query checking**: SQLx verifies all queries at build time

---

## Authentication and Authorization

### JWT-Based Authentication

```
1. Client -> nginx -> auth-service: POST /v1/auth/login {email, password}
2. Auth Service validates credentials (Argon2 hash comparison)
3. Generate JWT tokens: access (15 min) + refresh (7 days)
4. Client stores tokens, sends Authorization: Bearer <token>
5. nginx passes the header through; each backend service validates JWT independently
6. Client -> nginx -> auth-service: POST /v1/auth/refresh for token renewal
```

For WebSocket: JWT is validated during the connection upgrade in realtime-service.

### Authorization Patterns

- **Resource ownership**: Users can only modify their own profiles
- **Relationship-based**: Friends-only visibility for online status
- **Role-based (guild-service)**: Bitflag permission system with role hierarchy. Guild owners, admins, and custom roles with granular permissions (manage channels, kick members, etc.)

---

## Technology Stack

### Backend Services

```
Language:        Rust (stable, 1.75+)
Edge Proxy:      nginx (REST routing, TLS, rate limiting, CORS)
Web Framework:   Axum 0.7 (async, Tower middleware)
gRPC:            Tonic 0.11 (Protocol Buffers, HTTP/2, streaming)
Database:        PostgreSQL 16 (SQLx 0.8, compile-time checked)
Cache:           Redis 7 (session, presence, message cache, connection state)
Messaging:       NATS (JetStream for persistence, at-least-once delivery)
Auth:            jsonwebtoken 9.3, Argon2 password hashing
Real-time:       WebSocket (tokio-tungstenite) via realtime-service, WebRTC (voice)
API Docs:        utoipa 5 (OpenAPI/Swagger)
```

### Observability

```
Logging:   tracing + tracing-subscriber (structured, JSON output)
Metrics:   Prometheus + Grafana (request duration, error rates, throughput)
Health:    /health/live (liveness), /health/ready (readiness)
nginx:     Access logs, upstream response time tracking
```

### Infrastructure

```
Edge Proxy:    nginx (reverse proxy, TLS, rate limiting)
Containers:    Docker + Docker Compose (development)
CI/CD:         GitHub Actions (test, lint, build)
Future:        Kubernetes (autoscaling, service discovery, rolling updates)
```

---

## Deployment Architecture

### Development (Docker Compose)

```
docker-compose.yml provides:
  - nginx (port 80)
  - PostgreSQL 16 (port 5432)
  - Redis 7 (port 6379)
  - NATS (port 4222, monitoring 8222)
  - Prometheus (port 9090)
  - Grafana (port 3000)

Rust services run natively via cargo:
  make run-auth, make run-user, make run-realtime, etc.
```

### Production (Future -- Kubernetes)

- nginx Ingress Controller at the edge (or replace with cloud load balancer)
- Pod autoscaling per service based on CPU/request metrics
- Service discovery via Kubernetes DNS
- Rolling updates with health check gates
- Secrets management via Kubernetes Secrets / Vault

---

## Scalability Strategy

### Horizontal Scaling

All services are stateless:
- No session state in memory (JWT for auth, Redis for state)
- Any pod can handle any request
- Independent scaling per service based on load
- nginx distributes load across service replicas

realtime-service scales horizontally with Redis-backed connection state: any instance can look up which users are connected where.

### Database Scaling Path

1. **MVP**: Shared PostgreSQL, connection pooling
2. **Post-MVP**: Database per service, read replicas
3. **Scale**: PgBouncer, sharding by guild_id or user_id

### Performance Targets

| Metric | Target |
|--------|--------|
| p50 response time | <50ms |
| p95 response time | <100ms |
| p99 response time | <200ms |
| AI translation latency | <200ms (text), <500ms (voice) |
| Throughput per pod | 10k req/s |

---

## Monitoring and Observability

### Key Metrics

```
Service Health:
  - http_requests_total{method, status, service}
  - http_request_duration_seconds{method, path, service}
  - grpc_requests_total{method, status, service}
  - websocket_connections_active (realtime-service)

Business Metrics:
  - user_registrations_total
  - messages_sent_total
  - translations_completed_total
  - active_users_total

Infrastructure:
  - db_connections_active
  - cache_hits_total / cache_misses_total
  - nats_messages_published / nats_messages_consumed
  - nginx_requests_total, nginx_upstream_response_time
```

### Logging Standards

- Structured logging with `tracing`
- Request ID propagation across services (nginx generates, passes via header)
- Log levels: ERROR (immediate action), WARN (potentially harmful), INFO (operational), DEBUG/TRACE (development)

---

## Security Architecture

### Defense in Depth

```
+------------------------------------------+
| Layer 1: Network (Firewall, DDoS)       |
+------------------------------------------+
| Layer 2: nginx (TLS, rate limit, CORS)  |
+------------------------------------------+
| Layer 3: Authentication (JWT)           |
+------------------------------------------+
| Layer 4: Authorization (RBAC)           |
+------------------------------------------+
| Layer 5: Data (Encryption, Validation)  |
+------------------------------------------+
```

### Security Measures

- **nginx**: TLS 1.3 termination, rate limiting (`limit_req`), CORS headers, request size limits
- **Argon2** password hashing with salt
- **JWT** with short expiry (15min access, 7-day refresh with rotation)
- **SQL injection prevention**: SQLx compile-time checked parameterized queries
- **Input validation**: `validator` crate on all request DTOs
- **Rate limiting**: nginx `limit_req` at the edge, Governor crate on auth endpoints as defense-in-depth
- **Lint policy**: `unsafe_code` is forbidden, `unwrap_used`/`expect_used`/`panic` are denied at workspace level
- **Internal network**: Backend services only accessible from nginx and each other, not exposed to the internet directly
