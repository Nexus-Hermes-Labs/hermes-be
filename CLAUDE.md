# hermes-be — AI Agent Context

## First Rule: Read the Vault

Before scanning this codebase, read `../hermes-vault/` for project context. Start with `../hermes-vault/README.md`.

For backend-specific context:
- Service details: `../hermes-vault/02-services/{service-name}.md`
- Code patterns: `../hermes-vault/03-domain/patterns.md`
- Event contracts: `../hermes-vault/03-domain/event-contracts.md`
- Outbox pattern: `../hermes-vault/03-domain/outbox-pattern.md`

## Backend Conventions

- **Rust Cargo workspace** with 14 crates (12 services + 2 shared)
- Every service follows **6-layer DDD**: domain → application → infrastructure → presentation → state → bootstrap
- Dependencies injected via **AppBuilder** (no DI framework)
- All dependencies wrapped in `Arc<T>` for async task sharing
- **`thiserror`** for error types, **`anyhow`** only in bootstrap/main
- Error hierarchy: `DomainError → ApplicationError → ApiError` with `From` conversions
- **No `unwrap`/`expect`/`panic`** — denied at workspace level
- **SQLx compile-time queries** — `DATABASE_URL` must point to a live DB during build
- **Transactional Outbox** for event publishing (not inline NATS publish)
- **`RequestUser`** extractor reads identity from Traefik-injected headers

## Files You Should Never Read

- `Cargo.lock` — 6000+ lines of dependency versions, zero useful context
- `target/` — build artifacts
- `postman/` — large Postman JSON; use Swagger UI instead (`localhost:808x/swagger-ui`)
- `infra/grafana/provisioning/dashboards/*.json` — machine-generated
- `.venv/`, `reports/`, `coverage/` — transient

When exploring services, start from `src/domain/` (business rules) or `src/presentation/http/routes/` (endpoint wiring). The vault already summarizes each service's purpose, endpoints, and domain model — read `../hermes-vault/02-services/{name}.md` before diving into source.

## Lint Policy

```
unsafe_code = "forbid"
unwrap_used, expect_used, panic, dbg_macro = "deny"
clippy::all, pedantic, nursery, cargo = "warn"
```

## Testing

- Unit tests inline in `src/` modules
- Integration tests in `tests/` using `testcontainers` (real Postgres, Redis, NATS)
- `TestHarness::new()` in `tests/common/setup.rs` wires the full stack
- When adding migrations: update both `migrations/` AND `setup.rs` constants

## Common Commands

```bash
cargo check --workspace
cargo test --workspace           # requires Docker infra up
cargo test -p auth-service       # single service
make ci                          # fmt-check + clippy + tests
make lint                        # clippy -D warnings
make db-migrate                  # run all migrations
make sqlx-prepare                # regenerate .sqlx/ for offline builds
```
