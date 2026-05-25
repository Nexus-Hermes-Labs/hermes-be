# Hermes - Development Roadmap

**Last Updated:** May 25, 2026

## Table of Contents

- [Vision](#vision)
- [Development Phases](#development-phases)
- [Phase 1: MVP Core](#phase-1-mvp-core)
- [Phase 2: Real-time and Supporting](#phase-2-real-time-and-supporting)
- [Phase 3: AI Innovation](#phase-3-ai-innovation)
- [Phase 4: Scale and Polish](#phase-4-scale-and-polish)
- [Service Completion Matrix](#service-completion-matrix)
- [Technical Debt](#technical-debt)
- [Risk Register](#risk-register)

---

## Vision

Hermes is a real-time communication platform differentiated by AI-powered translation.
Development is organized into four phases, each building on the last:

1. **MVP Core** — Functional chat platform (auth, users, guilds, channels, messaging, realtime, Traefik)
2. **Real-time and Supporting** — Presence, media uploads, notifications
3. **AI Innovation** — Real-time text translation, then speech-to-text, then text-to-speech
4. **Scale and Polish** — Full-text search, voice channels, moderation, performance tuning

---

## Development Phases

```
Phase 1 (MVP Core)            Phase 2 (Real-time)      Phase 3 (AI)         Phase 4 (Scale)
+--------------------+        +-----------------+      +---------------+    +---------------+
| Traefik (infra)  ✅|        | presence-service|      | ai-service    |    | search-service|
| auth-service     ✅|        | media-service   |      |  text xlat    |    | voice-service |
| user-service     ✅|        | notification-svc|      |  STT          |    | moderation    |
| guild-service    ✅|        +-----------------+      |  TTS          |    +---------------+
| channel-service  ✅|                                  +---------------+
| chat-service     🔧|
| messaging-service🔧|
| realtime-service 🔧|
+--------------------+
  ✅ = complete  🔧 = in progress  ⬜ = not started
```

---

## Phase 1: MVP Core

A functional communication platform with authentication, user management, guild/channel structure, messaging, WebSocket event delivery, and Traefik as the edge proxy.

### Traefik — COMPLETE

Traefik serves as the edge proxy, replacing a custom API gateway.

Implemented:
- REST reverse proxy routing (`/v1/auth/*` → auth-service, etc.)
- TLS termination (HTTPS at edge, HTTP internally)
- Rate limiting at edge (100 req/s average, 50 burst)
- JWT validation via `ForwardAuth` middleware — delegates to `auth-service/internal/verify`, injects `X-User-Id`, `X-User-Role`, `X-User-Email` headers on success
- CORS configuration
- gzip compression
- WebSocket upgrade proxying to realtime-service (no ForwardAuth — token validated on WS handshake)
- Upstream health checks

What Traefik does NOT do:
- JWT logic (delegated entirely to auth-service via ForwardAuth)
- Event fanout (realtime-service's job)
- gRPC routing (services talk directly)

### auth-service (port 8081) — COMPLETE

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | Complete |
| Presentation | Complete |
| Tests | Complete |

Implemented endpoints:
- `POST /v1/auth/register` — User registration with Argon2 hashing
- `POST /v1/auth/login` — JWT token pair generation (access: 6h, refresh: 30d)
- `POST /v1/auth/refresh` — Token refresh with rotation
- `POST /v1/auth/logout` — Session invalidation
- `POST /v1/auth/verify-email` — Email verification
- `GET /internal/verify` — ForwardAuth endpoint called by Traefik on every protected request

### user-service (port 8082) — COMPLETE

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | Complete |
| Presentation | Complete |
| Tests | Complete |

Covers: user profiles, friend system, blocks, privacy settings, discriminators, gRPC server for cross-service user lookups.

### guild-service (port 8086) — COMPLETE

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | Complete |
| Presentation | Complete |
| Tests | Complete |

Covers: guild CRUD, role management, bitflag-based permission system, member management, invite system with expiry and usage limits.

### channel-service (port 8083) — COMPLETE

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | Complete |
| Presentation | Complete |
| Tests | Complete |

Covers: text/voice channels, categories, channel-level permission overrides. Uses gRPC to verify guild permissions via guild-service.

### chat-service (port 8084) — IN PROGRESS

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | In progress |
| Presentation | In progress |
| Tests | Partial |

Covers: messages, reactions, attachments, message history, direct messages. Publishes `hermes.message.*` events to NATS consumed by realtime-service.

Remaining work:
- Remaining HTTP endpoints and DTOs
- Full NATS event publishing for all message operations
- Comprehensive test coverage

### messaging-service (port 8093) — IN PROGRESS

| Layer | Status |
|---|---|
| Domain | Complete |
| Application | Complete |
| Infrastructure | In progress |
| Presentation | In progress |
| Tests | Partial |

Covers: message delivery orchestration, reaction events. Publishes to `MESSAGING_EVENTS` stream via Transactional Outbox.

### realtime-service (port 8092) — IN PROGRESS

| Layer | Status |
|---|---|
| WebSocket endpoint | In progress |
| Client registry | In progress |
| Subscription management | In progress |
| NATS → WebSocket bridge | In progress |
| Typing indicator fanout | Not started |
| Heartbeat / keepalive | Not started |

Covers: WebSocket connection management, NATS event fanout to connected clients, connection state in Redis.

Key design:
- `GET /ws?token=<access_token>` — upgrades connection, validates JWT on handshake
- `ClientRegistry`: `DashMap<UserId, Vec<WsSender>>` (one user, many tabs/devices)
- `SubscriptionRegistry`: `DashMap<ChannelId, HashSet<UserId>>`
- NATS subjects bridged: `hermes.message.created`, `hermes.message.updated`, `hermes.message.deleted`, `hermes.reaction.added`, `hermes.reaction.removed`
- Client opcodes: `SUBSCRIBE`, `UNSUBSCRIBE`, `TYPING_START`, `HEARTBEAT_ACK`

---

## Phase 2: Real-time and Supporting

Services that enhance the MVP with supporting infrastructure.

### presence-service (port 8087)

Scope: online/offline/idle/DND status tracking, typing indicators, last seen timestamps.

Implementation approach:
- Redis-backed presence state (ephemeral by nature)
- NATS subscription for realtime-service connect/disconnect events
- gRPC interface for other services to query presence

### media-service (port 8088)

Scope: file uploads, image processing (resize, thumbnails), CDN proxy, avatar management.

Implementation approach:
- Multipart upload endpoint via Axum (behind Traefik)
- Image processing pipeline (thumbnail generation, format conversion)
- Storage abstraction (local filesystem for dev, S3-compatible for production)

### notification-service (port 8089)

Scope: push notifications, unread message counts, @mention tracking, notification preferences.

Implementation approach:
- NATS consumer for message and mention events
- Per-user notification state in PostgreSQL
- Unread count aggregation
- Push notification delivery (FCM/APNs integration, future)

---

## Phase 3: AI Innovation

The differentiating feature: real-time AI-powered translation.

### ai-service (port 8091)

Development is split into three sub-phases:

**Phase 3a — Text Translation:**
- Language detection on incoming messages
- Translation via external AI model API (configurable provider)
- Translations stored alongside original messages
- Per-guild and per-user language preferences
- NATS pipeline: `message.created` → ai-service → `translation.completed`

**Phase 3b — Voice STT (Speech-to-Text):**
- Audio stream ingestion from voice-service
- Real-time transcription with language detection
- Translated subtitles pushed via realtime-service to clients
- Streaming API for low-latency results

**Phase 3c — Voice TTS (Text-to-Speech):**
- Translated text synthesized to speech
- Audio output streamed to target users in their preferred language
- Full speech-to-speech pipeline: speak → STT → translate → TTS → deliver
- Latency target: <500ms end-to-end

---

## Phase 4: Scale and Polish

Services for discoverability, voice communication, and platform maturity.

### search-service (port 8090)

Scope: full-text search across messages, users, and guilds.

Implementation approach:
- PostgreSQL full-text search for MVP (tsvector/tsquery)
- Optional Elasticsearch/Meilisearch integration for scale
- Search indexing via NATS event consumption
- Permission-aware results (only return content the user can access)

### voice-service (port 8085)

Scope: WebRTC P2P signaling, voice channel management, audio routing.

Implementation approach:
- Signaling server for WebRTC peer connection setup
- STUN/TURN server coordination
- Voice channel state management (who's in which channel)
- Integration with ai-service for real-time voice translation (Phase 3b/3c)

### Moderation Features

Cross-cutting concern, implemented as extensions to existing services:
- Message content filtering (chat-service)
- User reporting system (user-service)
- Auto-moderation rules (guild-service)
- Audit logging (all services via NATS events)

---

## Service Completion Matrix

| Service | Domain | Application | Infrastructure | Presentation | Tests | Overall |
|---|---|---|---|---|---|---|
| **Traefik** | N/A | N/A | Complete | N/A | N/A | **Complete** |
| **auth-service** | Done | Done | Done | Done | Done | **Complete** |
| **user-service** | Done | Done | Done | Done | Done | **Complete** |
| **guild-service** | Done | Done | Done | Done | Done | **Complete** |
| **channel-service** | Done | Done | Done | Done | Done | **Complete** |
| **chat-service** | Done | Done | In progress | In progress | Partial | **~70%** |
| **messaging-service** | Done | Done | In progress | In progress | Partial | **In progress** |
| **realtime-service** | Done | In progress | In progress | In progress | Partial | **~50%** |
| **presence-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **media-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **notification-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **ai-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **search-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **voice-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |

---

## Technical Debt

### Current Debt Items

| Item | Severity | Impact | Notes |
|---|---|---|---|
| realtime-service typing indicator + heartbeat | High | Blocks MVP completion | Remaining WS features |
| chat-service HTTP endpoints incomplete | High | Blocks MVP completion | Remaining handlers and DTOs |
| No integration tests for gRPC flows | Medium | Quality risk | Cross-service gRPC communication untested end-to-end |
| Shared database for all services | Low | Scale risk | Acceptable for MVP, plan migration post-MVP |
| No Redis caching layer in use | Low | Performance risk | Infrastructure wired, caching logic not yet applied |

### Planned Improvements

**Near-term (completing Phase 1):**
- Finish realtime-service WebSocket bridge (typing, heartbeat)
- Complete chat-service HTTP endpoints and NATS publishing
- Add integration tests for all gRPC communication paths
- Implement health check endpoints on all active services

**Post-MVP:**
- Split to database-per-service
- Activate Redis caching (sessions, profiles, relationships)
- Load testing and performance profiling under real workloads

---

## Risk Register

### Active Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| AI translation latency too high for real-time use | Medium | High | Streaming APIs, model selection, caching frequent translations |
| Database contention with shared DB at scale | Low | High | Monitor query performance, migrate to per-service DBs before bottleneck |
| Voice + AI pipeline complexity | Medium | Medium | Phase 3c is stretch goal, deliver text translation first |
| Scope creep across 12 services | Medium | Medium | Strict phase boundaries, MVP-first approach |

### Resolved Risks

| Risk | Resolution |
|---|---|
| Auth complexity | JWT + Argon2 + ForwardAuth working well |
| Domain modeling complexity | DDD / hexagonal architecture approach successful |
| Bidirectional relationship sync | PostgreSQL triggers handle it cleanly |
| API gateway complexity | Replaced with Traefik — simpler, battle-tested, ForwardAuth handles JWT centrally |
| gRPC integration complexity | Trait-based wrappers keep Tonic types out of application layer |
| User-service completion | Fully implemented including gRPC server and all HTTP endpoints |
| Guild/channel-service | Both fully implemented with bitflag RBAC and permission verification |
