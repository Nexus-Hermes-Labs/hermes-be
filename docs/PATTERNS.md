# Hermes - Code Patterns

**Version:** 1.0.0
**Last Updated:** March 25, 2026

This document describes the canonical code patterns used inside every service. It is the reference for developers writing or reviewing code. For system-level architecture (services, communication, deployment) see [ARCHITECTURE.md](../../ARCHITECTURE.md).

## Table of Contents

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
- [19. Transactional Outbox + Idempotent Consumer](#19-transactional-outbox--idempotent-consumer)

---

## 1. Layered DDD (Hexagonal Architecture)

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

## 2. Dependency Injection via Builder Pattern

Services are wired together manually inside `bootstrap/app_builder.rs` using a fluent builder. There is no DI container or framework magic.

```rust
// bootstrap/app_builder.rs
AppBuilder::new(config, db_pool, redis)
    .build_repositories()          // Arc<dyn SomeRepository>
    .build_domain_services()       // Arc<PasswordService>, Arc<JwtManager>
    .build_application_services()  // Arc<AuthService>
    .build_grpc_clients()
    .build_state()                 // AppState { ... }
```

All dependencies are wrapped in `Arc<T>` for shared ownership across async tasks. The construction order is explicit: infrastructure → repositories → domain services → application services → state.

---

## 3. Repository Pattern with Generic Base Trait

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

## 4. Unit of Work Pattern

Multi-step writes that must succeed or fail atomically are coordinated by a `UnitOfWork`. This solves the problem of sharing a `sqlx::Transaction` across multiple repository calls in async code.

The shared building blocks live in `common::infrastructure::persistence::unit_of_work`:

- `UnitOfWork` — base trait every per-service UoW extends, defining the consume-self `commit`/`rollback` contract.
- `UowFuture<'a>` / `UowCallback<'a, U>` — type aliases for the closure-based transaction API.
- `run_in_transaction(uow, op)` — runs the closure, commits on `Ok`, rolls back on `Err`. Per-service factory traits delegate their default `transaction()` impl to this helper.

```rust
// common::infrastructure::persistence::unit_of_work
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError>;
}

pub async fn run_in_transaction<U: UnitOfWork + ?Sized>(
    uow: Box<U>,
    operation: UowCallback<'_, U>,
) -> Result<(), RepositoryError> { ... }
```

Each service's UoW trait extends `UnitOfWork` and adds writer accessors. The factory exposes only `begin()`; `transaction()` has a default impl forwarding to `run_in_transaction`:

```rust
// auth-service: application/ports/unit_of_work.rs
#[async_trait]
pub trait AuthUnitOfWork: UnitOfWork {
    fn credentials(&self) -> &dyn CredentialWriter;
    fn sessions(&self) -> &dyn SessionWriter;
    fn outbox(&self) -> &dyn OutboxWriter;
}

#[async_trait]
pub trait AuthUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn AuthUnitOfWork>, RepositoryError>;

    async fn transaction(
        &self,
        operation: UowCallback<'_, dyn AuthUnitOfWork>,
    ) -> Result<(), RepositoryError> {
        let uow = self.begin().await?;
        run_in_transaction(uow, operation).await
    }
}
```

Application services always call `transaction()` — manual `begin()` / `commit()` is a code smell because it forces the caller to remember rollback on every error path:

```rust
self.uow_factory
    .transaction(Box::new(move |uow| {
        Box::pin(async move {
            uow.credentials().update(&credential).await?;
            uow.sessions().save(&session).await?;
            uow.outbox().save(&event).await?;
            Ok(())
        })
    }))
    .await?;
```

The Postgres impl uses a shared `SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>` (from `common::infrastructure::outbox`) so every sub-writer can borrow the same transaction concurrently. Dropping the UoW without calling `commit` rolls the transaction back automatically — `run_in_transaction` relies on that safety net.

---

## 5. Aggregate and Value Object Patterns

Domain entities are modeled as DDD aggregates with a single aggregate root. Value objects enforce invariants at construction time — once constructed, they are always valid.

```rust
// auth-service: domain/auth_credential/entity.rs
pub struct AuthCredential {
    pub id: Uuid,                    // aggregate root ID
    pub email: Email,                // value object — always valid
    pub password_hash: PasswordHash,
    pub status: AccountStatus,
    pub role: SystemRole,
}

impl AuthCredential {
    pub fn new(email: Email, hash: PasswordHash) -> Self { ... }  // creation
    pub fn from_persisted(...) -> Self { ... }                    // reconstruction
}

// Value object — private constructor, validated on construction
pub struct Email(String);
impl Email {
    pub fn new(raw: &str) -> Result<Self, DomainError> {
        // validates format, returns Err if invalid
    }
}
```

Permission bitfields are also value objects using `i64` with named constants and `grant()`, `revoke()`, `has()` methods (`guild-service: domain/guild_role/valueobject.rs`).

---

## 6. Application Service (Orchestration)

Application services live in `application/services/` and are the only layer that coordinates repositories, domain services, events, and transactions together. Handlers in the presentation layer call these and do nothing else.

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

## 7. Domain Events with Event Envelope

Domain events are wrapped in a generic `EventEnvelope<T>` that carries metadata alongside the payload. Events are published to NATS after successful operations.

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

Published to NATS subjects like `user.created`, `message.created`. `realtime-service` fans events out to connected WebSocket clients.

---

## 8. Trait-Based Infrastructure Abstractions

All external infrastructure is hidden behind async traits. This makes application services testable and decouples them from specific implementations (NATS, Postgres, etc.).

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

Other abstracted traits: `PasswordService`, `TokenHasher`, `UserProfileClient` (gRPC wrapper), `EmailService`. In tests, `mockall` generates mock implementations of these traits.

---

## 9. Hierarchical Error Handling

Errors follow a strict three-layer hierarchy. Each layer has its own error type, and `From` impls convert upward. `unwrap` and `expect` are denied by the workspace lint policy.

```
DomainError        (business rule violations)
      ↓ From
ApplicationError   (use-case failures)
      ↓ From
ApiError           (mapped to HTTP status codes + JSON response)
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

impl IntoResponse for ApiError { ... } // → JSON { error, code } + status code
impl From<AuthApplicationError> for ApiError { ... }
```

All error propagation uses `?` with proper `From` conversions. `thiserror` is used to derive `Error` impls.

---

## 10. Gateway Authentication Pattern

JWT authentication is centralized at Traefik via `ForwardAuth`. Backend services never verify JWTs — they receive pre-validated user identity as injected headers, which `RequestUser` reads via `FromRequestParts`.

```
Client  Authorization: Bearer <jwt>
  │
  ▼
Traefik  ForwardAuth → GET auth-service/internal/verify
                            ↓ validates JWT (JwtManager)
                            ↓ 401 if invalid/expired
                            ↓ 200 + injects X-User-Id, X-User-Role, X-User-Email
  │
  ▼
Backend service
  │
  ▼  RequestUser (implements FromRequestParts)
     reads X-User-Id  → Uuid
     reads X-User-Role → SystemRole
     reads X-User-Email → String
```

```rust
// common/src/middleware/authentication.rs
pub struct RequestUser {
    pub id: Uuid,
    pub role: SystemRole,
    pub email: String,
}

// Used in any handler across any service:
pub async fn create_guild(
    State(state): State<AppState>,
    RequestUser { id: owner_id, .. }: RequestUser,
    Json(req): Json<CreateGuildRequest>,
) -> Result<(StatusCode, Json<CreateGuildResponse>), ApiError> { ... }
```

Authorization extractors follow the same `FromRequestParts` pattern:

```rust
// common/src/middleware/authorization.rs
pub struct AdminOnly;         // 403 if role != Admin
pub struct ModeratorOrAbove;  // 403 if role < Moderator
```

For the Traefik-level flow diagram, see [ARCHITECTURE.md — Authentication Flow](../../ARCHITECTURE.md#authentication-flow).

---

## 11. DTO Pattern with Compile-Time Validation

Request and response types are separate structs in `presentation/http/dto/`. Request DTOs carry `#[validate(...)]` annotations enforced before reaching handlers. Domain types are never exposed directly in HTTP responses.

```rust
// chat-service: presentation/dto/message/request.rs
#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
    pub reply_to_id: Option<Uuid>,
}
```

`ToSchema` derives OpenAPI schema automatically. `Validate` enforces rules at the boundary.

---

## 12. State Composition Root

Each service has a layered `AppState` that groups related services under named sub-states. Passed to Axum via `State<AppState>` and accessed in handlers.

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

## 13. Service Bootstrap Pattern

Every service follows the same startup sequence in `bootstrap/mod.rs` and `main.rs`. This creates a predictable initialization order with no surprise side effects.

```
main.rs
  1. init_config(service_name)       // OnceCell singleton
  2. init_tracing(config.logging)    // structured logging
  3. connect_db(config.database)     // PgPool
  4. connect_redis(config.redis)     // ConnectionManager
  5. init_metrics()                  // Prometheus
  6. AppBuilder::new(...).build()    // full DI wiring
  7. spawn background tasks          // tokio::spawn
  8. start HTTP server (Axum)        // tokio::spawn
  9. start gRPC server (Tonic)       // tokio::spawn
 10. await shutdown signal           // SIGTERM / Ctrl+C
```

HTTP and gRPC run concurrently on separate ports (e.g., HTTP :8081, gRPC :50081).

---

## 14. Pagination Pattern

Pagination is standardized in `common` with clamping to prevent abuse and a consistent response envelope.

```rust
// common/src/pagination.rs
pub struct PaginationParams {
    pub page: u32,       // min 1, auto-clamped
    pub page_size: u32,  // 1..=100, auto-clamped
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

## 15. Background Task Pattern

Long-running periodic jobs are spawned as Tokio tasks and respect the graceful shutdown signal.

```rust
// auth-service: application/background/email_verification_cleanup_task.rs
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

## 16. Graceful Shutdown Pattern

A `tokio::sync::watch` channel broadcasts the shutdown signal. All long-running tasks (background jobs, HTTP server, gRPC server) hold a receiver and exit cleanly when signaled.

```rust
// bootstrap/mod.rs
let (shutdown_tx, shutdown_rx) = watch::channel(false);

tokio::spawn(async move {
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = unix_sigterm() => {},
    }
    shutdown_tx.send(true).ok();
});

tokio::select! {
    _ = http_server  => {},
    _ = grpc_server  => {},
    _ = shutdown_rx.changed() => {},
}
```

---

## 17. OnceCell Configuration Singleton

Configuration is loaded once at startup and stored in a global `OnceCell`. Any layer can call `config()` to get the static reference without threading it through every function call.

```rust
// common-config/src/lib.rs
static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init_config(service_name: &str) -> Result<(), ConfigError> {
    // Load order: workspace .env → service .env → env vars (highest priority)
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
    pub secrets: SecretsConfig,
    pub grpc_endpoints: GrpcEndpointsConfig,
    pub smtp: SmtpConfig,
    pub logging: LoggingConfig,
}
```

---

## 18. Workspace-Level Lint Policy

The root `Cargo.toml` enforces strict lints across every crate. No per-service exceptions.

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used    = "deny"
expect_used    = "deny"
panic          = "deny"
dbg_macro      = "deny"
all            = "warn"
pedantic       = "warn"
nursery        = "warn"
cargo          = "warn"
```

All error propagation uses `?` with `From` conversions (`thiserror`). Panics are compile-time forbidden. Allowed exceptions: `module_name_repetitions`, `too_many_lines`, `missing_errors_doc`, `type_complexity`, `needless_pass_by_value`, `struct_excessive_bools`.

---

## 19. Transactional Outbox + Idempotent Consumer

Any service that publishes domain events to NATS JetStream writes them through the **Transactional Outbox** pattern instead of publishing inline. This guarantees the DB write and the event are committed atomically, and that any service downstream sees each event at-least-once with stable IDs.

The reusable building blocks live in `common::infrastructure::outbox`:

- `OutboxWriter` / `PgOutboxWriter` — transactional writer for `outbox_events`. Composed into every per-service `UnitOfWork` so application services enqueue events in the same SQLx transaction as their aggregate writes.
- `OutboxRepository` — fetches publishable rows scoped to a `source_service`, applies exponential backoff on failure (`POWER(2, retry_count) s`, capped at 1h).
- `OutboxPublisherTask` — periodic `BackgroundTask` that drains the outbox to JetStream, stamping each event with `Nats-Msg-Id = event_id` so JetStream rejects duplicate publishes within the stream's `duplicate_window`.
- `OutboxStreamConfig` + `ensure_stream` — shared stream config (retention, max-age, duplicate window) used by both publisher and consumer so the two sides cannot drift.
- `JetStreamEventHandler` / `JetStreamConsumerRunner` — trait-based idempotent consumer. The runner unwraps the `EventEnvelope`, inserts `event_id` into `processed_events` inside a transaction, calls the handler with the same transaction, and ack's on success. Duplicates are detected by the `processed_events` `event_id` primary key and skipped.
- `ephemeral_fanout_consumer` — helper for realtime fan-out (e.g. `realtime-service`) where missed events are acceptable. Creates an ephemeral pull consumer with `deliver_policy: New` and `inactive_threshold: 60s`.

### Publisher side (writer + worker)

```rust
// application service: enqueue event in the same transaction as the aggregate write
let outbox = NewOutboxEvent {
    id: envelope.event_id,
    aggregate_id: message.id(),
    aggregate_type: "chat_message".into(),
    event_type: "chat.message.created".into(),
    payload: serde_json::to_value(&envelope)?,
};

uow_factory.transaction(Box::new(move |uow| {
    Box::pin(async move {
        uow.messages().save(&message).await?;
        uow.outbox().save(&outbox).await?;
        Ok(())
    })
})).await?;

// External side-effects (email, gRPC notifications) run AFTER commit.
```

```rust
// bootstrap: spawn the publisher worker
let repository = Arc::new(OutboxRepository::new(db_pool, "chat-service"));
let stream = OutboxStreamConfig::new("CHAT_EVENTS", vec!["chat.>".into()]);
let task = OutboxPublisherTask::new("chat-outbox-publisher", repository, &nats_url, &stream).await?;
tokio::spawn(common::infrastructure::background::run_periodic_task(task, shutdown_rx));
```

### Consumer side (idempotent handler)

```rust
pub struct UserCreatedHandler;

#[async_trait]
impl JetStreamEventHandler for UserCreatedHandler {
    type Event = UserCreatedPayload;
    fn subject(&self) -> &str { "user.created" }
    fn durable_name(&self) -> &str { "user-service-user-created" }

    async fn handle(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        envelope: EventEnvelope<Self::Event>,
    ) -> Result<(), anyhow::Error> {
        sqlx::query("INSERT INTO user_profiles ...")
            .bind(envelope.payload.user_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

// bootstrap:
let runner = JetStreamConsumerRunner::new(db_pool, &nats_url, &stream_config, UserCreatedHandler).await?;
tokio::spawn(runner.run(shutdown_rx));
```

### Required schema

Every service that publishes events needs the `outbox_events` table (with `source_service` column to scope per-publisher fetches in shared databases); every service that consumes events needs `processed_events`:

```sql
CREATE TABLE outbox_events (
    id              UUID         PRIMARY KEY,
    aggregate_id    UUID         NOT NULL,
    aggregate_type  TEXT         NOT NULL,
    event_type      TEXT         NOT NULL,
    payload         JSONB        NOT NULL,
    source_service  TEXT         NOT NULL,
    status          TEXT         NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'published', 'failed')),
    retry_count     INTEGER      NOT NULL DEFAULT 0 CHECK (retry_count >= 0),
    last_error      TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    next_retry_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ
);
CREATE INDEX idx_outbox_events_publishable
    ON outbox_events (source_service, next_retry_at)
    WHERE status IN ('pending', 'failed');

CREATE TABLE processed_events (
    event_id     UUID         PRIMARY KEY,
    event_type   TEXT         NOT NULL,
    processed_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
```

### Subject and stream conventions

- One stream per domain: `USER_EVENTS` (`user.>`), `CHAT_EVENTS` (`chat.>`), `MESSAGING_EVENTS` (`messaging.>`).
- Event types double as JetStream subjects: `user.created`, `chat.message.created`, `messaging.reaction.added`. The publisher uses `event.event_type` directly — no mapping layer.
- Stream config (`retention: Limits`, `storage: File`, 7-day `max_age`, 2-hour `duplicate_window`) is shared via `OutboxStreamConfig` so publisher and consumer never disagree.

### When to use which consumer

- **Durable** state changes (e.g. user-service materialising user profiles from `user.created`): `JetStreamEventHandler` + `JetStreamConsumerRunner`. Requires `processed_events` and a stable `durable_name`. At-least-once delivery with idempotency at the DB layer.
- **Ephemeral fan-out** (e.g. realtime-service pushing events to WebSocket clients): `ephemeral_fanout_consumer`. No durability, no idempotency — clients reconnect and lose missed events the same way they would with core pub/sub.

### Why not inline NATS publish?

A direct `nats.publish` after `repo.save` has three failure modes:
1. DB commits, NATS publish fails — downstream consumers never see the event.
2. NATS publish succeeds, DB transaction rolls back — phantom event referencing a row that doesn't exist.
3. Process crashes between commit and publish — same as (1).

The outbox eliminates all three because the DB write and the event are in the same transaction, and a separate worker is responsible for the unreliable network step.
