# Hermes - System Architecture

**Version:** 3.0.0
**Last Updated:** March 25, 2026

## Table of Contents

- [Overview](#overview)
- [Architecture Goals](#architecture-goals)
- [Service Map](#service-map)
- [Internal Architecture Patterns](#internal-architecture-patterns)
  - [1. Layered DDD (Hexagonal Architecture)](#1-layered-ddd-hexagonal-architecture)
  - [2. Dependency Injection via Builder Pattern](#2-dependency-injection-via-builder-pattern)
  - [3. Repository Pattern with Generic Base Trait](#3-repository-pattern-with-generic-base-trait)
  - [4. Unit of Work Pattern](#4-unit-of-work-pattern)
  - [5. Aggregate and Value Object Patterns](#5-aggregate-and-value-object-patterns)
  - [6. Application Service (Orchestration)](#6-application-service-orchestration)
  - [7. Domain Events with Event Envelope](#7-domain-events-with-event-envelope)
  - [8. Trait-Based Infrastructure Abstractions](#8-trait-based-infrastructure-abstractions)
  - [9. Hierarchical Error Handling](#9-hierarchical-error-handling)
  - [10. Gateway Authentication Pattern](#10-gateway-authentication-pattern)
  - [11. DTO Pattern with Compile-Time Validation](#11-dto-pattern-with-compile-time-validation)
  - [12. State Composition Root](#12-state-composition-root)
  - [13. Service Bootstrap Pattern](#13-service-bootstrap-pattern)
  - [14. Pagination Pattern](#14-pagination-pattern)
  - [15. Background Task Pattern](#15-background-task-pattern)
  - [16. Graceful Shutdown Pattern](#16-graceful-shutdown-pattern)
  - [17. OnceCell Configuration Singleton](#17-oncecell-configuration-singleton)
  - [18. Workspace-Level Lint Policy](#18-workspace-level-lint-policy)
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

## Internal Architecture Patterns

Each service in the codebase follows a consistent set of patterns. These are the canonical patterns used throughout the system.

---

### 1. Layered DDD (Hexagonal Architecture)

Every service is organized into six distinct layers. Dependencies only flow inward — infrastructure knows about domain, domain knows about nothing.

```
services/{service-name}/src/
├── domain/           # Pure business logic. No external dependencies.
│   ├── entity.rs     # Aggregate roots and entities
│   ├── valueobject.rs
│   ├── repository.rs # Repository traits (interfaces only)
│   └── error.rs      # Domain-level errors
├── application/      # Orchestration. Calls domain + infrastructure ports.
│   ├── services/     # Application service impls
│   ├── events/       # Domain event definitions
│   ├── ports/        # Unit of Work traits, external service traits
│   └── background/   # Periodic background tasks
├── infrastructure/   # Implements domain ports.
│   ├── persistence/  # SQLx repository implementations
│   ├── grpc/         # gRPC client wrappers
│   ├── messaging/    # NATS publisher
│   └── security/     # Argon2, SHA256, JWT
├── presentation/     # Protocol handlers.
│   ├── http/         # Axum routes, handlers, DTOs
│   └── grpc/         # Tonic gRPC server
├── state/            # AppState composition
└── bootstrap/        # Wiring + server startup
```

**Why**: Keeps domain logic free of infrastructure concerns. Repository traits are defined in `domain/`, their SQL implementations are in `infrastructure/`. Swapping the database never touches domain code.

---

### 2. Dependency Injection via Builder Pattern

Services are wired together manually inside `bootstrap/app_builder.rs` using a fluent builder pattern. There is no DI container or framework magic.

```rust
// bootstrap/app_builder.rs
AppBuilder::new(config, db_pool, redis)
    .build_repositories()       // Arc<dyn SomeRepository>
    .build_domain_services()    // Arc<PasswordService>, Arc<JwtManager>
    .build_application_services() // Arc<AuthService>
    .build_grpc_clients()
    .build_state()              // AppState { ... }
```

All dependencies are wrapped in `Arc<T>` for shared ownership across async tasks. The construction order is explicit: infrastructure → repositories → domain services → application services → state.

---

### 3. Repository Pattern with Generic Base Trait

A generic `Repository<T, ID>` trait in `common` defines standard CRUD operations. Domain-specific repositories extend it with query methods relevant to their aggregate.

```rust
// common/src/infrastructure/persistence/repository.rs
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, DbError>;
    async fn save(&self, entity: &T) -> Result<(), DbError>;
    async fn delete(&self, id: ID) -> Result<(), DbError>;
    async fn exists(&self, id: ID) -> Result<bool, DbError>;
}

// auth-service: domain/auth_credential/repository.rs
#[async_trait]
pub trait AuthCredentialRepository: Repository<AuthCredential, Uuid> {
    async fn find_by_email(&self, email: &Email) -> Result<Option<AuthCredential>, DbError>;
    async fn set_verification_token(&self, ...) -> Result<(), DbError>;
}
```

Traits are defined in `domain/`, implemented in `infrastructure/persistence/postgres/`. Application services depend only on the trait — the SQL implementation is injected via `Arc<dyn AuthCredentialRepository>`.

---

### 4. Unit of Work Pattern

Multi-step writes that must succeed or fail atomically are coordinated by a `UnitOfWork`. This solves the problem of sharing a `sqlx::Transaction` across multiple repository calls in async code.

```rust
// Shared transaction handle passed to sub-writers
type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

// guild-service: infrastructure/persistence/postgres/unit_of_work.rs
pub struct PgGuildUnitOfWork {
    tx: SharedTx,
    pub guild_writer: PgGuildWriter,       // holds clone of SharedTx
    pub member_writer: PgGuildMemberWriter,
    pub invite_writer: PgGuildInviteWriter,
}

impl PgGuildUnitOfWork {
    pub async fn commit(self) -> Result<(), DbError> { ... } // consumes self
    // Drop = automatic rollback if commit() never called
}
```

`UnitOfWork` traits are defined in `application/ports/`. Application services receive the factory via constructor injection.

---

### 5. Aggregate and Value Object Patterns

Domain entities are modeled as DDD aggregates with a single aggregate root. Value objects enforce invariants at construction time.

```rust
// auth-service: domain/auth_credential/entity.rs
pub struct AuthCredential {
    pub id: Uuid,                  // aggregate root ID
    pub email: Email,              // value object — always valid
    pub password_hash: PasswordHash,
    pub status: AccountStatus,
    pub role: SystemRole,
}

impl AuthCredential {
    pub fn new(email: Email, hash: PasswordHash) -> Self { ... }   // creation
    pub fn from_persisted(...) -> Self { ... }                     // reconstruction
}

// Value object — private constructor, validated on construction
pub struct Email(String);
impl Email {
    pub fn new(raw: &str) -> Result<Self, DomainError> {
        // validates format, returns Err if invalid
    }
}
```

Permission bitfields are also value objects using `i64` with named constants and `grant()`, `revoke()`, `has()` methods.

---

### 6. Application Service (Orchestration)

Application services sit in `application/services/` and are the only layer that coordinates repositories, domain services, events, and transactions together. Handlers in the presentation layer call these and do nothing else.

```rust
// auth-service: application/services/authentication/service.rs
pub struct AuthService {
    credential_repo: Arc<dyn AuthCredentialRepository>,
    session_repo: Arc<dyn AuthSessionRepository>,
    password_service: Arc<dyn PasswordService>,
    jwt_manager: Arc<JwtManager>,
    event_publisher: Arc<dyn EventPublisher>,
    // ...
}

impl AuthService {
    pub async fn register(&self, cmd: RegisterCommand) -> Result<AuthTokens, AuthError> {
        // 1. validate email uniqueness (repo)
        // 2. hash password (domain service)
        // 3. create aggregate
        // 4. persist (repo)
        // 5. publish UserCreatedEvent (event publisher)
        // 6. return tokens
    }
}
```

---

### 7. Domain Events with Event Envelope

Domain events are wrapped in a generic `EventEnvelope<T>` that carries metadata alongside the payload. Published to NATS after successful operations.

```rust
// common/src/domain/event.rs
pub struct EventEnvelope<T: DomainEvent> {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub version: u32,
    pub occurred_at: DateTime<Utc>,
    pub source_service: String,
    pub correlation_id: Option<Uuid>,  // for distributed tracing
    pub payload: T,
}

// auth-service: application/events/user_created.rs
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub email: String,
    pub role: SystemRole,
}
```

Events are published to NATS subjects like `user.created`, `message.created`. Other services subscribe asynchronously. `realtime-service` fans events out to connected WebSocket clients.

---

### 8. Trait-Based Infrastructure Abstractions

All external infrastructure is hidden behind async traits. This makes services testable and decouples application logic from specific implementations (NATS, Postgres, etc.).

```rust
// common/src/infrastructure/messaging/event_publisher.rs
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_bytes(&self, subject: &str, payload: Vec<u8>) -> Result<(), MessagingError>;
}

pub trait EventPublisherExt: EventPublisher {
    async fn publish<T: DomainEvent>(&self, envelope: EventEnvelope<T>) -> Result<(), MessagingError>;
}
```

Other abstracted traits: `PasswordService`, `TokenHasher`, `UserProfileClient` (gRPC wrapper), `EmailService`.

---

### 9. Hierarchical Error Handling

Errors follow a strict three-layer hierarchy. Each layer has its own error type, and `From` impls convert upward. No `unwrap` or `expect` anywhere (denied by lint policy).

```
DomainError        (business rule violations)
      ↓ From
ApplicationError   (use-case failures)
      ↓ From
ApiError           (mapped to HTTP status codes)
```

```rust
// presentation/http/error.rs
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError { ... } // maps to JSON { error, code } + status code

impl From<AuthApplicationError> for ApiError { ... }
```

---

### 10. Gateway Authentication Pattern

JWT authentication is centralized at the Traefik edge via `ForwardAuth`. Backend services never verify JWTs themselves — they receive pre-validated user identity via injected headers.

```
Client
  │  Authorization: Bearer <jwt>
  ▼
Traefik
  │  ForwardAuth → GET http://auth-service:8081/internal/verify
  │                     ↓ validates JWT with JwtManager
  │                     ↓ 401 if invalid/expired
  │                     ↓ 200 + X-User-Id, X-User-Role, X-User-Email if valid
  ▼
Backend Service
  │  receives request with identity headers already set
  ▼
RequestUser extractor (implements FromRequestParts)
  │  reads X-User-Id → Uuid
  │  reads X-User-Role → SystemRole
  │  reads X-User-Email → String
  ▼
Handler parameter
```

```rust
// common/src/middleware/authentication.rs
pub struct RequestUser {
    pub id: Uuid,
    pub role: SystemRole,
    pub email: String,
}

// Usage in any handler across any service:
pub async fn create_guild(
    State(state): State<AppState>,
    RequestUser { id: owner_id, .. }: RequestUser,
    Json(req): Json<CreateGuildRequest>,
) -> Result<(StatusCode, Json<CreateGuildResponse>), ApiError> { ... }
```

Authorization extractors follow the same pattern:

```rust
// common/src/middleware/authorization.rs
pub struct AdminOnly;     // implements FromRequestParts — 403 if not Admin
pub struct ModeratorOrAbove; // 403 if not Moderator or Admin
```

---

### 11. DTO Pattern with Compile-Time Validation

Request and response types are separate structs in `presentation/http/dto/`. Request DTOs carry `#[validate(...)]` annotations that are enforced before reaching handlers.

```rust
// chat-service: presentation/dto/message/request.rs
#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
    pub reply_to_id: Option<Uuid>,
}
```

`ToSchema` derives OpenAPI schema for free. `Validate` enforces rules at the boundary. Domain types are never exposed directly in HTTP responses.

---

### 12. State Composition Root

Each service has a layered `AppState` that groups related services under named sub-states. This is passed to Axum via `State<AppState>` and accessed in handlers.

```rust
// guild-service: state/mod.rs
pub struct AppState {
    pub guild: GuildState,
    pub shared: SharedState,
}

pub struct GuildState {
    pub guild_service: Arc<GuildService>,
    pub member_service: Arc<GuildMemberService>,
    pub role_service: Arc<GuildRoleService>,
    pub invite_service: Arc<GuildInviteService>,
}

pub struct SharedState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub metrics: Metrics,
}
```

---

### 13. Service Bootstrap Pattern

Every service follows the same startup sequence in `bootstrap/mod.rs` and `main.rs`. This creates a predictable initialization order with no surprise side effects.

```
main.rs
  1. load_config(service_name)         // OnceCell singleton
  2. init_tracing(config.logging)      // structured logging
  3. connect_db(config.database)       // PgPool
  4. connect_redis(config.redis)       // ConnectionManager
  5. init_metrics()                    // Prometheus
  6. AppBuilder::new(...).build()      // DI wiring
  7. spawn background tasks            // tokio::spawn
  8. start HTTP server (Axum)          // tokio::spawn
  9. start gRPC server (Tonic)         // tokio::spawn
 10. await shutdown signal             // SIGTERM / Ctrl+C
```

---

### 14. Pagination Pattern

Pagination is standardized in `common` with clamping to prevent abuse and a consistent response envelope.

```rust
// common/src/pagination.rs
pub struct PaginationParams {
    pub page: u32,       // min 1, clamped
    pub page_size: u32,  // 1..=100, clamped
}

impl PaginationParams {
    pub fn offset(&self) -> u32 { (self.page - 1) * self.page_size }
}

pub struct Paginated<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
```

All list endpoints return `Paginated<T>`. Query params are extracted via Axum `Query<PaginationParams>`.

---

### 15. Background Task Pattern

Long-running periodic jobs are spawned as Tokio tasks and respect the shutdown broadcast signal.

```rust
// auth-service: application/background/email_verification_cleanup_task.rs
pub struct EmailVerificationCleanupTask { ... }

impl EmailVerificationCleanupTask {
    pub async fn run(self, mut shutdown: watch::Receiver<bool>) {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(3600)) => {
                    self.credential_repo.delete_expired_tokens().await.ok();
                }
                _ = shutdown.changed() => break,
            }
        }
    }
}

// bootstrap/mod.rs
tokio::spawn(cleanup_task.run(shutdown_rx.clone()));
```

---

### 16. Graceful Shutdown Pattern

A `tokio::sync::watch` channel broadcasts a shutdown signal. All long-running tasks (background jobs, HTTP server, gRPC server) receive the receiver and exit cleanly when signaled.

```rust
// bootstrap/mod.rs
let (shutdown_tx, shutdown_rx) = watch::channel(false);

// OS signal handler
tokio::spawn(async move {
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = unix_sigterm() => {},
    }
    shutdown_tx.send(true).ok();
});

// All spawned tasks hold a clone of shutdown_rx
tokio::select! {
    _ = http_server => {},
    _ = grpc_server => {},
    _ = shutdown_rx.changed() => {},
}
```

---

### 17. OnceCell Configuration Singleton

Configuration is loaded once at startup and stored in a global `OnceCell`. Any code in any layer can call `config()` to get the static reference without passing config through every function call.

```rust
// common-config/src/lib.rs
static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init_config(service_name: &str) -> Result<(), ConfigError> {
    // Load order: workspace .env → service .env → env vars
    CONFIG.set(Config::load_and_validate(service_name)?).ok();
    Ok(())
}

pub fn config() -> &'static Config { CONFIG.get().expect("config not initialized") }
```

Config shape:

```rust
pub struct Config {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub redis: CacheConfig,
    pub nats: MessagingConfig,
    pub secrets: SecretsConfig,       // JWT secrets, etc.
    pub grpc_endpoints: GrpcEndpointsConfig,
    pub smtp: SmtpConfig,
    pub logging: LoggingConfig,
}
```

---

### 18. Workspace-Level Lint Policy

The root `Cargo.toml` enforces strict lints across every crate in the workspace. No exceptions per service.

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
dbg_macro = "deny"
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
```

This means all error propagation uses `?` with proper `From` conversions, and panics are compile-time forbidden.

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
