# Hermes

A real-time communication platform with AI-powered translation, built in Rust.

Hermes enables seamless cross-language communication through real-time text translation, voice-to-text transcription, and speech synthesis. Built as a distributed microservices system using Domain-Driven Design, it combines the functionality of modern chat platforms with breakthrough AI translation capabilities.

## The Innovation: Real-Time AI Translation

Most communication platforms treat language as a barrier users must solve themselves. Hermes integrates translation at the infrastructure level:

- **Text Translation** — Messages are translated in real-time as they're sent, with the original preserved alongside translations for each participant's language
- **Voice Subtitles** — Speech-to-text transcription with live translation, displayed as subtitles during voice calls
- **Voice Dubbing** — Full speech-to-speech translation: speak in your language, others hear it in theirs (Phase 3)

This turns every guild and channel into a multilingual space by default, not as an afterthought.

## Architecture Overview

Traefik sits at the edge handling REST routing, TLS termination, rate limiting, and centralized JWT validation via ForwardAuth. Behind Traefik, 12 backend services handle domain logic. A dedicated realtime-service manages WebSocket connections and NATS event fanout independently of Traefik.

```
                    Clients
                       |
                +------+------+
                |   Traefik   |  REST routing, ForwardAuth JWT, rate limiting
                +------+------+
                       |
       +-------+-------+-------+-------+
       |       |       |       |       |
       v       v       v       v       v
    +------+ +----+ +-----+ +----+ +----+   ... other services
    | Auth | |User| |Guild| |Chan| |Chat|
    | 8081 | |8082| |8086 | |8083| |8084|
    +------+ +----+ +-----+ +----+ +----+

                    Clients
                       |
                  WebSocket
                       |
                +------+------+
                |  Realtime   |  WebSocket + NATS fanout
                |    8080     |
                +------+------+
                       |
                  +----+----+
                  |  NATS   |
                  +---------+
```

## Service Table

| Service | Port | Purpose | Phase | Status |
|---|---|---|---|---|
| **Traefik** | 80 / 8080 | REST proxy, ForwardAuth JWT validation, rate limiting | Infrastructure | ✅ Complete |
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

## Tech Stack

| Layer | Technology |
|---|---|
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
| Observability | Prometheus + Grafana, tracing |
| Testing | testcontainers, mockall, fake, rstest |
| Infrastructure | Docker Compose |

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
hermes-be/
+-- services/
|   +-- common/              # Shared library (models, errors, utilities, middleware)
|   |   +-- common-config/   # Configuration loading (OnceCell singleton)
|   |   +-- migrations/      # Database migrations
|   +-- auth-service/        # Authentication (complete ✅)
|   +-- user-service/        # User management (complete ✅)
|   +-- guild-service/       # Guild management (complete ✅)
|   +-- channel-service/     # Channel management (complete ✅)
|   +-- chat-service/        # Messaging (in progress 🔧)
|   +-- realtime-service/    # WebSocket + event fanout (in progress 🔧)
|   +-- presence-service/    # Online status (stub ⬜)
|   +-- media-service/       # File uploads (stub ⬜)
|   +-- notification-service/# Notifications (stub ⬜)
|   +-- search-service/      # Full-text search (stub ⬜)
|   +-- ai-service/          # AI translation (stub ⬜)
|   +-- voice-service/       # Voice signaling (stub ⬜)
+-- proto/                   # Protocol Buffer definitions
+-- infra/
|   +-- postgres/            # Database init scripts
|   +-- prometheus/          # Metrics config
|   +-- grafana/             # Dashboards
+-- docs/
|   +-- ARCHITECTURE.md      # System design, service responsibilities, data architecture
|   +-- PATTERNS.md          # Code patterns used inside every service
|   +-- ROADMAP.md           # Phased development plan with completion tracking
+-- docker-compose.yml
+-- Cargo.toml               # Workspace configuration
```

## Communication Patterns

- **Traefik** (edge): REST routing, ForwardAuth JWT validation via auth-service, rate limiting, CORS
- **HTTP REST** (client-facing): Axum with JSON. Clients hit Traefik, which validates the JWT, injects `X-User-Id`/`X-User-Role`/`X-User-Email` headers, then forwards to the service.
- **gRPC** (service-to-service): Tonic with Protocol Buffers. Direct between services, not through Traefik. Compile-time codegen via `build.rs`.
- **NATS** (async events): Cross-service notifications, AI translation pipeline.
- **WebSocket** (real-time): Clients connect directly to realtime-service. NATS events are fanned out to connected clients.

## Roadmap

| Phase | Focus | Status |
|---|---|---|
| Phase 1 | MVP Core (auth, users, guilds, channels, chat, realtime, Traefik) | 🔧 In progress |
| Phase 2 | Real-time and Supporting (presence, media, notifications) | ⬜ Not started |
| Phase 3 | AI Innovation (text translation, STT, TTS) | ⬜ Not started |
| Phase 4 | Scale and Polish (search, voice, moderation) | ⬜ Not started |

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design, service map, communication, data, deployment
- [Patterns](docs/PATTERNS.md) — 18 code patterns used inside every service (DDD layers, Repository, Unit of Work, error handling, etc.)
- [Roadmap](docs/ROADMAP.md) — Phased development plan with per-service completion tracking

## License

MIT License — see [LICENSE](LICENSE) for details.
