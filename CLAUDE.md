# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hermes is a Discord-like real-time communication platform built as a Rust microservices system. Currently in early development (auth-service complete, user-service ~60%, other services not yet started).

## Common Commands

```bash
# Setup & infrastructure
make setup               # Initial setup (docker up, migrate, seed)
make up / make down      # Start/stop Docker services (PostgreSQL, Redis, NATS, Prometheus, Grafana)
make dev                 # Full dev environment (docker + migrate + seed)

# Build
cargo build --workspace              # Build all
cargo build -p auth-service          # Build single service
cargo check --workspace --all-targets # Quick check

# Test
cargo test --workspace               # All tests
cargo test -p auth-service           # Single service tests
cargo test --workspace -- --nocapture # With output

# Code quality
cargo fmt --all                                                      # Format
cargo clippy --workspace --all-targets --all-features -- -D warnings # Lint
make ci                                                              # Format check + lint + test

# Database
make db-migrate          # Run migrations (from services/common/migrations/)
make db-seed             # Seed dev data
make db-reset            # Clean + setup + migrate + seed
make db-shell            # psql into hermes database

# Run individual services
make run-auth            # cargo run --bin auth-service
make run-user            # cargo run --bin user-service

# Protobuf
make proto-generate      # Build services to trigger tonic codegen
```

## Architecture

### Workspace Structure

Cargo workspace with 9 crates: 7 services + `common` (shared models/errors/utilities) + `common-config` (configuration loading).

Services: `auth-service` (8081), `user-service` (8082), `channel-service` (8083), `chat-service` (8084), `voice-service` (8085), `presence-service` (8087), `gateway-service` (8080).

### DDD Layer Pattern (per service)

Each service follows Domain-Driven Design with these layers:
- **`domain/`** - Pure business logic: entities, value objects, repository traits, domain services, errors
- **`application/`** - Use case orchestration: application services, events
- **`infrastructure/`** - External concerns: PostgreSQL repository implementations, gRPC clients/servers, messaging
- **`presentation/`** - HTTP handlers (Axum routes, DTOs, middleware) and gRPC service definitions
- **`state/`** - Application state (shared across handlers)
- **`bootstrap/`** - Service initialization and wiring

### Communication

- **HTTP REST** (client-facing): Axum with JSON, JWT bearer auth
- **gRPC** (service-to-service): Tonic with Protocol Buffers in `proto/` directory. Proto codegen happens at build time via `build.rs`.
- **NATS** (async events): Post-MVP, for cross-service notifications

### Database

Single shared PostgreSQL database (MVP). All migrations in `services/common/migrations/`. SQLx with compile-time query verification.

Config via `.env` (copy from `.env.example`). Key: `DATABASE_URL=postgres://hermes:hermes@localhost:5432/hermes`.

## Lint Policy

Strict clippy configuration in workspace `Cargo.toml`:
- `unsafe_code` is **forbidden**
- `unwrap_used`, `expect_used`, `panic`, `dbg_macro` are **denied** — use proper error handling (`thiserror`/`anyhow`)
- `clippy::all`, `pedantic`, `nursery`, `cargo` are all warnings
- Allowed exceptions: `module_name_repetitions`, `too_many_lines`, `missing_errors_doc`, `type_complexity`, `needless_pass_by_value`, `struct_excessive_bools`

## Key Tech Stack

- **Web**: Axum 0.7, Tokio 1.38
- **gRPC**: Tonic 0.11, Prost 0.12
- **DB**: SQLx 0.8 (PostgreSQL, compile-time checked)
- **Auth**: jsonwebtoken 9.3, Argon2 password hashing
- **Cache**: Redis 0.25
- **Messaging**: async-nats 0.33
- **API Docs**: utoipa 5 (OpenAPI/Swagger)
- **Testing**: testcontainers, mockall, fake, rstest
