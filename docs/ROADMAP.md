# Hermes - Development Roadmap

**Last Updated:** February 16, 2026

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
The development is organized into four phases, each building on the last:

1. **MVP Core** -- Functional chat platform (auth, users, guilds, channels, messaging, realtime, nginx)
2. **Real-time and Supporting** -- Presence, media uploads, notifications
3. **AI Innovation** -- Real-time text translation, then speech-to-text, then text-to-speech
4. **Scale and Polish** -- Full-text search, voice channels, moderation, performance tuning

---

## Development Phases

```
Phase 1 (MVP Core)           Phase 2 (Real-time)      Phase 3 (AI)         Phase 4 (Scale)
+-------------------+        +-----------------+      +---------------+    +---------------+
| auth-service   [X]|        | presence-service|      | ai-service    |    | search-service|
| user-service   [~]|        | media-service   |      |  text xlat    |    | voice-service |
| guild-service  [ ]|        | notification-svc|      |  STT          |    | moderation    |
| channel-service[ ]|        +-----------------+      |  TTS          |    +---------------+
| chat-service   [ ]|                                  +---------------+
| realtime-service[]|
| nginx (infra)  [ ]|
+-------------------+
  [X] = complete  [~] = in progress  [ ] = not started
```

---

## Phase 1: MVP Core

A functional communication platform with authentication, user management, guild/channel structure, messaging, WebSocket event delivery, and nginx as the edge proxy.

### nginx -- NOT STARTED

nginx serves as the edge proxy, replacing a custom API gateway.

Scope:
- REST reverse proxy routing (`/v1/auth/*` -> auth-service, etc.)
- TLS termination (HTTPS at edge, HTTP internally)
- Rate limiting (`limit_req` zones per endpoint category)
- CORS configuration
- gzip compression
- WebSocket upgrade proxying to realtime-service
- Upstream health checks

What nginx does NOT do:
- JWT validation (each service handles this)
- Event fanout (realtime-service's job)
- gRPC routing (services talk directly)

### auth-service (port 8081) -- COMPLETE

| Layer | Status |
|-------|--------|
| Domain | Complete |
| Application | Complete |
| Infrastructure | Complete |
| Presentation | Complete |
| Tests | Complete |

Implemented endpoints:
- `POST /v1/auth/register` -- User registration with Argon2 hashing
- `POST /v1/auth/login` -- JWT token pair generation
- `POST /v1/auth/refresh` -- Token refresh with rotation
- `POST /v1/auth/logout` -- Session invalidation
- `POST /v1/auth/verify-email` -- Verification email

### user-service (port 8082) -- IN PROGRESS (~60%)

| Layer | Status |
|-------|--------|
| Domain | Complete (User, UserRelationship entities, value objects, domain services) |
| Application | Complete (UserApplicationService, UserRelationshipApplicationService, events) |
| Infrastructure | In progress (repositories, gRPC server/client) |
| Presentation | Not started (HTTP endpoints, DTOs) |
| Tests | Partial (integration tests for profile management) |

Remaining work:
- PostgreSQL repository implementations (User, UserRelationship, Discriminator)
- gRPC service definitions and server implementation
- HTTP endpoints (18 endpoints: profile, friends, blocks)
- Comprehensive test coverage

### guild-service (port 8086) -- NOT STARTED

Scope: guilds, roles, members, invites, permission management (bitflag-based).

Key domain concepts:
- Guild entity with owner, settings, icon
- Role entity with bitflag permissions and hierarchy
- Member entity linking users to guilds with roles
- Invite entity with expiry and usage limits

### channel-service (port 8083) -- NOT STARTED

Scope: text/voice channels, categories, channel-level permission overrides.

Depends on guild-service for permission verification via gRPC.

### chat-service (port 8084) -- NOT STARTED

Scope: messages, reactions, attachments, message history, direct messages.

Key features:
- Channel messages and direct messages
- Message editing and deletion
- Reactions
- Attachment metadata (actual files handled by media-service)
- Pagination and history retrieval

### realtime-service (port 8080) -- NOT STARTED

Scope: WebSocket connection management, NATS event fanout, connection state tracking.

Key features:
- WebSocket lifecycle (connect with JWT auth, heartbeat, disconnect)
- NATS subscription per connected user (guild events, DM events, etc.)
- Event fanout to the correct connected clients
- Connection state tracked in Redis (for horizontal scaling)

This service does NOT handle REST routing (nginx does that) or rate limiting (nginx does that). Its single responsibility is real-time event delivery over WebSocket.

---

## Phase 2: Real-time and Supporting

Services that enhance the MVP with real-time features and supporting infrastructure.

### presence-service (port 8087)

Scope: online/offline/idle/DND status tracking, typing indicators, last seen timestamps.

Implementation approach:
- Redis-backed presence state (ephemeral by nature)
- NATS subscription for realtime-service connect/disconnect events
- gRPC interface for other services to query presence

### media-service (port 8088)

Scope: file uploads, image processing (resize, thumbnails), CDN proxy, avatar management.

Implementation approach:
- Multipart upload endpoint via Axum (behind nginx)
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

**Phase 3a -- Text Translation:**
- Language detection on incoming messages
- Translation via external AI model API (configurable provider)
- Translations stored alongside original messages
- Per-guild and per-user language preferences
- NATS pipeline: `message.created` -> ai-service -> `translation.completed`

**Phase 3b -- Voice STT (Speech-to-Text):**
- Audio stream ingestion from voice-service
- Real-time transcription with language detection
- Translated subtitles pushed via realtime-service to clients
- Streaming API for low-latency results

**Phase 3c -- Voice TTS (Text-to-Speech):**
- Translated text synthesized to speech
- Audio output streamed to target users in their preferred language
- Full speech-to-speech pipeline: speak -> STT -> translate -> TTS -> deliver
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
|---------|--------|-------------|----------------|--------------|-------|---------|
| **nginx** | N/A | N/A | Not started | N/A | N/A | **Not started** |
| **auth-service** | Done | Done | Done | Done | Done | **Complete** |
| **user-service** | Done | Done | In progress | Not started | Partial | **~60%** |
| **guild-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **channel-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **chat-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
| **realtime-service** | Not started | Not started | Not started | Not started | Not started | **Stub** |
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
|------|----------|--------|-------|
| User-service missing HTTP endpoints | High | Blocks MVP completion | 18 endpoints need handlers + DTOs |
| User-service missing repository impls | High | Blocks HTTP endpoints | PostgreSQL repos for User, UserRelationship, Discriminator |
| No integration tests for gRPC flows | Medium | Quality risk | Auth <-> User gRPC communication untested end-to-end |
| No observability beyond basic logging | Medium | Ops risk | Need Prometheus metrics, health checks on all services |
| No nginx configuration | Medium | Blocks full MVP deployment | Need routing config, TLS, rate limiting |
| Shared database for all services | Low | Scale risk | Acceptable for MVP, plan migration post-MVP |
| No Redis caching layer | Low | Performance risk | Add after core functionality works |

### Planned Improvements

**Near-term (during Phase 1):**
- Complete user-service infrastructure and presentation layers
- Add integration tests for all gRPC communication paths
- Implement health check endpoints on all active services
- Set up nginx configuration with routing for all MVP services

**Post-MVP:**
- Split to database-per-service
- Add Redis caching (sessions, profiles, relationships)
- Implement NATS event bus for cross-service communication
- Structured logging with request ID propagation (nginx-generated)

---

## Risk Register

### Active Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| gRPC integration complexity across 12 services | Medium | High | Start with core flows, expand incrementally |
| AI translation latency too high for real-time use | Medium | High | Streaming APIs, model selection, caching frequent translations |
| Database contention with shared DB at scale | Low | High | Monitor query performance, migrate to per-service DBs before bottleneck |
| Voice + AI pipeline complexity | Medium | Medium | Phase 3c is stretch goal, deliver text translation first |
| Scope creep across 12 services | Medium | Medium | Strict phase boundaries, MVP-first approach |

### Resolved Risks

| Risk | Resolution | Date |
|------|-----------|------|
| Auth complexity | JWT + Argon2 working well | Phase 1 |
| Domain modeling complexity | DDD approach successful | Phase 1 |
| Bidirectional relationship sync | PostgreSQL triggers handle it | Phase 1 |
| API gateway complexity | Replaced with nginx, simpler and battle-tested | Phase 1 |
