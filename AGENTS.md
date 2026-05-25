# hermes-be — AI Agent Context (Codex / Generic Agents)

## First Rule: Read the Vault

Before scanning this codebase, read `../hermes-vault/` for context. Start with `../hermes-vault/README.md`.

Backend-specific vault files:
- `../hermes-vault/02-services/{service-name}.md` — per-service details
- `../hermes-vault/03-domain/patterns.md` — code patterns (DDD, UoW, errors)
- `../hermes-vault/03-domain/event-contracts.md` — NATS event subjects and payloads
- `../hermes-vault/03-domain/outbox-pattern.md` — transactional outbox

## Conventions

- Rust Cargo workspace, 14 crates
- 6-layer DDD: domain → application → infrastructure → presentation → state → bootstrap
- `thiserror` for errors, `anyhow` only in bootstrap
- Error chain: DomainError → ApplicationError → ApiError (with `From` impls)
- `unsafe_code` forbidden, `unwrap`/`panic` denied
- SQLx compile-time query checking
- Transactional Outbox for event publishing
- testcontainers for integration tests (real DB, not mocks)
- `RequestUser` extractor reads identity from Traefik ForwardAuth headers

## Commands

```bash
cargo check --workspace
cargo test --workspace
cargo test -p auth-service
make ci
make db-migrate
make sqlx-prepare
```
