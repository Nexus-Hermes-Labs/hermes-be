# Hermes

A real-time communication platform with AI-powered translation, built in Rust.

Hermes enables seamless cross-language communication through real-time text translation, voice-to-text transcription, and speech synthesis. Built as a distributed microservices system using Domain-Driven Design, it combines the functionality of modern chat platforms with breakthrough AI translation capabilities.

## The Innovation: Real-Time AI Translation

Most communication platforms treat language as a barrier users must solve themselves. Hermes integrates translation at the infrastructure level:

- **Text Translation** -- Messages are translated in real-time as they're sent, with the original preserved alongside translations for each participant's language
- **Voice Subtitles** -- Speech-to-text transcription with live translation, displayed as subtitles during voice calls
- **Voice Dubbing** -- Full speech-to-speech translation: speak in your language, others hear it in theirs (future)

This turns every guild and channel into a multilingual space by default, not as an afterthought.

## Architecture Overview

Hermes uses Traefik as the edge proxy for REST routing, ForwardAuth JWT validation, and rate limiting. Behind Traefik, 12 backend services handle domain logic, and a dedicated realtime-service manages WebSocket connections and event fanout.

```
                    Clients
                       |
                +------+------+
                |   Traefik   |  (REST routing, ForwardAuth, rate limit)
                +------+------+
                       |
       +-------+-------+-------+-------+
       |       |       |       |       |
       v       v       v       v       v
    +------+ +----+ +-----+ +----+ +----+   ...other services
    | Auth | |User| |Guild| |Chan| |Chat|
    | 8081 | |8082| |8086 | |8083| |8084|
    +------+ +----+ +-----+ +----+ +----+

                    Clients
                       |
                  WebSocket
                       |
                +------+------+
                |  Realtime   |  (WebSocket, NATS fanout)
                |    8080     |
                +------+------+
                       |
                  +----+----+
                  |  NATS   |
                  +---------+
```

### Service Table

| Service | Port | Purpose | Phase | Status |
|---------|------|---------|-------|--------|
| **Traefik** | 80 (dashboard 8080) | REST reverse proxy, ForwardAuth JWT validation, rate limiting | Infrastructure | ✅ Complete |
| **auth-service** | 8081 | Authentication, JWT, sessions | MVP | ✅ Complete |
| **user-service** | 8082 | Profiles, relationships, privacy | MVP | ✅ Complete |
| **guild-service** | 8086 | Guilds, roles, members, invites, permissions | MVP | ✅ Complete |
| **channel-service** | 8083 | Text and voice channels, categories | MVP | ✅ Complete |
| **chat-service** | 8084 | Messages, reactions, attachments, history | MVP | 🔧 In progress |
| **realtime-service** | 8080 | WebSocket connections, NATS event fanout | MVP | 🔧 In progress |
| **presence-service** | 8087 | Online status, typing indicators | Phase 2 | ⬜ Not started |
| **media-service** | 8088 | File uploads, image processing, CDN proxy | Phase 2 | ⬜ Not started |
| **notification-service** | 8089 | Push notifications, unreads, mentions | Phase 2 | ⬜ Not started |
| **ai-service** | 8091 | Real-time translation (text + voice STT/TTS) | Phase 3 | ⬜ Not started |
| **search-service** | 8090 | Full-text search (messages, users, guilds) | Phase 4 | ⬜ Not started |
| **voice-service** | 8085 | WebRTC P2P signaling, voice channels | Phase 4 | ⬜ Not started |

### Service Map

```
Phase 1 (MVP Core)              Phase 2 (Real-time)         Phase 3 (AI)        Phase 4 (Scale)
 +-----------------+             +------------------+        +--------------+    +----------------+
 | auth-service ✅  |             | presence-service |        | ai-service   |    | search-service |
 | user-service ✅  |             | media-service    |        |  - text xlat |    | voice-service  |
 | guild-service ✅ |             | notification-svc |        |  - STT       |    +----------------+
 | channel-service✅|             +------------------+        |  - TTS       |
 | chat-service 🔧  |                                         +--------------+
 | realtime-svc 🔧  |
 | Traefik (infra) ✅ |
 +-----------------+
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (stable) |
| Edge Proxy | Traefik v3 (ForwardAuth, rate limiting, routing) |
| Web Framework | Axum 0.7, Tower middleware |
| gRPC | Tonic 0.11, Protocol Buffers (Prost 0.12) |
| Database | PostgreSQL 16 (SQLx 0.8, compile-time checked) |
| Cache | Redis 7 |
| Messaging | NATS (async-nats 0.33) |
| Auth | JWT (jsonwebtoken 9.3), Argon2 password hashing |
| Real-time | WebSocket (Tokio Tungstenite), WebRTC (voice) |
| API Docs | utoipa 5 (OpenAPI/Swagger) |
| API Testing | Schemathesis (property-based, stateful) |
| Observability | Prometheus + Grafana, tracing |
| Testing | testcontainers, mockall, fake, rstest |
| Infrastructure | Docker Compose |

## API Quality & Testing

All MVP REST APIs are **battle-tested** using [Schemathesis](https://schemathesis.io/), a property-based API testing tool that auto-generates test cases directly from OpenAPI schemas.

### What Schemathesis covers

- **Schema conformance** -- every endpoint validated against its OpenAPI spec
- **Edge case fuzzing** -- boundary values, unexpected inputs, and malformed payloads
- **Stateful testing** -- multi-step flows (e.g. register → login → create guild → invite member)
- **Error handling** -- ensures 5xx responses never leak from unexpected inputs

### Services validated

| Service | OpenAPI Spec | Schemathesis Status |
|---------|-------------|---------------------|
| auth-service | ✅ | ✅ Battle-tested |
| user-service | ✅ | ✅ Battle-tested |
| guild-service | ✅ | ✅ Battle-tested |
| channel-service | ✅ | ✅ Battle-tested |

## Quick Start

### Prerequisites

- Docker and Docker Compose
- `make`

### Setup

```bash
git clone https://github.com/bulutcan99/hermes.git
cd hermes
cp .env.example .env

# Start everything (Traefik + all backend services + infra)
make dev
```

All services are fully containerized and launched from the root `hermes` repo. Traefik starts as the edge proxy on port 80 (dashboard on port 8080), routing traffic to backend services.

### Development (local builds)

For contributors who want to build or test services locally without Docker:

```bash
# Prerequisites: Rust (stable, 1.75+), sqlx-cli
# cargo install sqlx-cli --no-default-features --features postgres

cargo check --workspace              # Quick check
cargo test --workspace               # Run all tests
cargo clippy --workspace --all-targets --all-features -- -D warnings  # Lint
make ci                              # Format check + lint + test
```

## Project Structure

```
hermes/
+-- services/
|   +-- common/              # Shared library (models, errors, utilities)
|   |   +-- common-config/   # Configuration loading
|   |   +-- migrations/      # Database migrations
|   +-- auth-service/        # Authentication (complete ✅)
|   +-- user-service/        # User management (complete ✅)
|   +-- guild-service/       # Guild management (complete ✅)
|   +-- channel-service/     # Channel management (complete ✅)
|   +-- chat-service/        # Messaging (in progress 🔧)
|   +-- voice-service/       # Voice signaling (stub)
|   +-- presence-service/    # Online status (stub)
|   +-- realtime-service/    # WebSocket + event fanout (in progress 🔧)
|   +-- media-service/       # File uploads (stub)
|   +-- notification-service/# Notifications (stub)
|   +-- search-service/      # Full-text search (stub)
|   +-- ai-service/          # AI translation (stub)
+-- proto/                   # Protocol Buffer definitions
+-- infra/
|   # Note: Traefik config lives in the root hermes repo (../infra/traefik/)
|   +-- postgres/             # Database init scripts
|   +-- prometheus/           # Metrics
|   +-- grafana/              # Dashboards
+-- docs/
|   +-- ARCHITECTURE.md      # System architecture
|   +-- ROADMAP.md           # Development roadmap
+-- docker-compose.yml
+-- Cargo.toml               # Workspace configuration
```

## DDD Layer Pattern

Each service follows Domain-Driven Design:

```
service/
+-- domain/          # Pure business logic: entities, value objects, repository traits, errors
+-- application/     # Use case orchestration: application services, events
+-- infrastructure/  # External concerns: PostgreSQL repos, gRPC clients, messaging
+-- presentation/    # HTTP handlers (Axum), gRPC service definitions
+-- state/           # Application state
+-- bootstrap/       # Service initialization and wiring
```

## Communication Patterns

- **Traefik** (edge): REST reverse proxy, ForwardAuth JWT validation via auth-service, rate limiting
- **HTTP REST** (client-facing): Axum with JSON, JWT bearer auth. Clients hit Traefik, which routes to backend services.
- **gRPC** (service-to-service): Tonic with Protocol Buffers, compile-time codegen via `build.rs`
- **NATS** (async events): Cross-service notifications, AI translation pipeline
- **WebSocket** (real-time): Clients connect to realtime-service for live event streaming. NATS events are fanned out to connected clients.

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full development roadmap.

| Phase | Focus | Status |
|-------|-------|--------|
| Phase 1 | MVP Core (auth, users, guilds, channels, chat, realtime, Traefik) | 🔧 In progress |
| Phase 1.5 | Frontend (web client) | 🆕 Starting now |
| Phase 2 | Real-time and Supporting (presence, media, notifications) | Not started |
| Phase 3 | AI Innovation (text translation, STT, TTS) | Not started |
| Phase 4 | Scale and Polish (search, voice, moderation) | Not started |

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) -- System design, service responsibilities, data architecture
- [Development Roadmap](docs/ROADMAP.md) -- Phased development plan with completion tracking

## License

MIT License -- see [LICENSE](LICENSE) for details.
