# Hermes - Code Patterns

**Version:** 1.0.0
**Last Updated:** March 25, 2026

This document describes the canonical code patterns used inside every service. It is the reference for developers writing or reviewing code. For system-level architecture (services, communication, deployment) see [ARCHITECTURE.md](./ARCHITECTURE.md).

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

```rust
// Shared transaction handle passed to sub-writers
type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

// guild-service: infrastructure/persistence/postgres/unit_of_work.rs
pub struct PgGuildUnitOfWork {
    tx: SharedTx,
    pub guild_writer: PgGuildWriter,        // holds clone of SharedTx
    pub member_writer: PgGuildMemberWriter,
    pub invite_writer: PgGuildInviteWriter,
}

impl PgGuildUnitOfWork {
    pub async fn commit(self) -> Result<(), DbError> { ... } // consumes self
    // Drop = automatic rollback if commit() was never called
}
```

`UnitOfWork` traits are defined in `application/ports/`. Application services receive the factory via constructor injection.

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

For the Traefik-level flow diagram, see [ARCHITECTURE.md — Authentication and Authorization](./ARCHITECTURE.md#authentication-and-authorization).

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
