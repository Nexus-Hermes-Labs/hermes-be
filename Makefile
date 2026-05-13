SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

# ──────────────────────────────────────────────────────────────────────────────
# Colors
# ──────────────────────────────────────────────────────────────────────────────
BLUE   := \033[0;34m
GREEN  := \033[0;32m
YELLOW := \033[1;33m
RED    := \033[0;31m
NC     := \033[0m

# ──────────────────────────────────────────────────────────────────────────────
# Service catalogs
# ──────────────────────────────────────────────────────────────────────────────
# Services with a docker-compose.yml of their own
DOCKER_SERVICES := auth user guild channel messaging chat realtime

# Services with a Postgres database (migrations / seeds / sqlx / DB_URL)
DB_SERVICES     := auth user guild channel messaging

# Services that bind a DATABASE_URL when running locally
RUN_DB_SERVICES := auth user guild channel messaging

# Services without a database (just `cargo run`)
RUN_PLAIN_SERVICES := chat realtime voice presence media notification search ai

# ──────────────────────────────────────────────────────────────────────────────
# Paths
# ──────────────────────────────────────────────────────────────────────────────
INFRA_COMPOSE := infra/docker-compose.yml
COMPOSE_FILE  = services/$(1)-service/docker-compose.yml
MIGRATION_DIR = services/$(1)-service/migrations
SEED_DIR      = services/$(1)-service/seeds/dev

# ──────────────────────────────────────────────────────────────────────────────
# Database URLs (host = localhost for cargo, hermes-postgres for docker-network)
# ──────────────────────────────────────────────────────────────────────────────
DB_URL          = postgres://hermes:hermes@localhost:5432/hermes_$(1)
DB_URL_NET      = postgres://hermes:hermes@hermes-postgres:5432/hermes_$(1)

DB_URL_AUTH      := $(call DB_URL,auth)
DB_URL_USER      := $(call DB_URL,user)
DB_URL_GUILD     := $(call DB_URL,guild)
DB_URL_CHANNEL   := $(call DB_URL,channel)
DB_URL_MESSAGING := $(call DB_URL,messaging)

# ──────────────────────────────────────────────────────────────────────────────
# Help
# ──────────────────────────────────────────────────────────────────────────────
.PHONY: help
help: ## Display this help message
	@echo -e "$(BLUE)Hermes - Microservices Platform$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make $(GREEN)<target>$(NC)\n"} \
		/^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-28s$(NC) %s\n", $$1, $$2 } \
		/^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) }' $(MAKEFILE_LIST)
	@echo ""
	@echo -e "$(YELLOW)Per-service targets (generated)$(NC)"
	@echo -e "  $(GREEN)up-<svc> down-<svc> restart-<svc> rebuild-<svc> logs-<svc>$(NC)  svc ∈ {$(DOCKER_SERVICES)}"
	@echo -e "  $(GREEN)db-migrate-<svc> db-seed-<svc> db-shell-<svc>$(NC)  svc ∈ {$(DB_SERVICES)}"
	@echo -e "  $(GREEN)sqlx-prepare-<svc> test-integration-<svc>$(NC)     svc ∈ {$(DB_SERVICES)}"
	@echo -e "  $(GREEN)run-<svc>$(NC)                                     svc ∈ {$(RUN_DB_SERVICES) $(RUN_PLAIN_SERVICES)}"

##@ Setup

.PHONY: install setup
install: ## Install development dependencies
	@echo -e "$(BLUE)Installing dependencies...$(NC)"
	@cargo install sqlx-cli --no-default-features --features postgres
	@cargo install cargo-watch
	@cargo install cargo-audit
	@echo -e "$(GREEN)Done$(NC)"

setup: up ## Initial project setup (infra up + migrate + seed)
	@echo -e "$(BLUE)Setting up project...$(NC)"
	@cp -n .env.example .env || true
	@$(MAKE) db-migrate
	@$(MAKE) db-seed
	@echo -e "$(GREEN)Setup complete! Edit .env if needed.$(NC)"

##@ Docker — Infrastructure

.PHONY: up down restart clean prune
up: ## Start infrastructure services (postgres, redis, nats, mailpit, monitoring)
	@echo -e "$(BLUE)Starting infrastructure...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) up -d --wait
	@sleep 3
	@echo -e "$(GREEN)Infrastructure ready$(NC)"

down: ## Stop infrastructure services
	@echo -e "$(YELLOW)Stopping infrastructure...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) down
	@echo -e "$(GREEN)Done$(NC)"

restart: down up ## Restart infrastructure

clean: ## Remove all containers, volumes, locally-built images, and networks
	@echo -e "$(RED)Cleaning up Docker resources...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) down -v --rmi local
	@for svc in $(DOCKER_SERVICES); do \
		docker-compose -f services/$$svc-service/docker-compose.yml down -v --rmi local 2>/dev/null || true; \
	done
	@docker volume rm $$(docker volume ls -q | grep hermes) 2>/dev/null || true
	@echo -e "$(BLUE)Pruning dangling images and build cache...$(NC)"
	@docker image prune -f >/dev/null
	@docker builder prune -f >/dev/null
	@echo -e "$(GREEN)Cleanup completed$(NC)"

prune: ## Deep clean: dangling images + builder cache (keeps running containers)
	@echo -e "$(BLUE)Pruning dangling images...$(NC)"
	@docker image prune -f
	@echo -e "$(BLUE)Pruning builder cache...$(NC)"
	@docker builder prune -f
	@echo -e "$(GREEN)Prune complete$(NC)"

##@ Docker — Services

# up-<svc> / down-<svc> / restart-<svc> / rebuild-<svc> generated for each docker service
define DOCKER_SERVICE_RULES
.PHONY: up-$(1) down-$(1) restart-$(1) rebuild-$(1)
up-$(1): ## Start $(1)-service container
	@echo -e "$$(BLUE)Starting $(1)-service...$$(NC)"
	@docker-compose -f $(call COMPOSE_FILE,$(1)) up -d --wait
	@echo -e "$$(GREEN)$(1)-service ready$$(NC)"

down-$(1): ## Stop $(1)-service container
	@docker-compose -f $(call COMPOSE_FILE,$(1)) down

restart-$(1): down-$(1) up-$(1) ## Restart $(1)-service container

# ETC: make rebuild-auth
rebuild-$(1): ## Rebuild image and restart $(1)-service container (use after code changes)
	@echo -e "$$(BLUE)Rebuilding $(1)-service...$$(NC)"
	@docker-compose -f $(call COMPOSE_FILE,$(1)) up -d --build --wait
	@docker image prune -f >/dev/null
	@echo -e "$$(GREEN)$(1)-service rebuilt and ready$$(NC)"
endef
$(foreach s,$(DOCKER_SERVICES),$(eval $(call DOCKER_SERVICE_RULES,$(s))))

.PHONY: up-all down-all
up-all: up $(addprefix up-,$(DOCKER_SERVICES)) ## Start infra + all service containers

down-all: $(addprefix down-,$(DOCKER_SERVICES)) down ## Stop all service containers + infra

##@ Logs

.PHONY: logs logs-postgres logs-redis logs-nats logs-grafana logs-prometheus
logs:            ## Show infra logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f
logs-postgres:   ## Show PostgreSQL logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f postgres
logs-redis:      ## Show Redis logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f redis
logs-nats:       ## Show NATS logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f nats
logs-grafana:    ## Show Grafana logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f grafana
logs-prometheus: ## Show Prometheus logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f prometheus

# logs-<svc> generated per docker service
define LOGS_RULE
.PHONY: logs-$(1)
logs-$(1): ## Show $(1)-service logs
	@docker-compose -f $(call COMPOSE_FILE,$(1)) logs -f $(1)-service
endef
$(foreach s,$(DOCKER_SERVICES),$(eval $(call LOGS_RULE,$(s))))

##@ Protobuf / gRPC

.PHONY: proto-generate proto-generate-auth proto-generate-user proto-clean
proto-generate: proto-generate-auth proto-generate-user ## Generate protobuf for all services
	@echo -e "$(GREEN)Done$(NC)"

proto-generate-auth: ## Generate protobuf for auth-service
	@cargo build -p auth-service

proto-generate-user: ## Generate protobuf for user-service
	@cargo build -p user-service

proto-clean: ## Clean all protobuf artifacts
	@cargo clean -p auth-service
	@cargo clean -p user-service

##@ Database — Migrations
# Migrations are embedded via sqlx::migrate!() and run automatically on service startup.
# These targets are kept for CI/CD environments with direct postgres access.

.PHONY: db-migrate
db-migrate: ## Migrations run automatically on service startup
	@echo -e "$(YELLOW)Migrations run automatically when services start.$(NC)"
	@echo -e "$(YELLOW)Use 'make up-all' to start services and apply migrations.$(NC)"

# db-migrate-<svc> via dockerised psql against hermes-network
define MIGRATE_RULE
.PHONY: db-migrate-$(1)
db-migrate-$(1): ## Run $(1)-service migrations via Docker network
	@echo -e "$$(BLUE)Running $(1)-service migrations...$$(NC)"
	@docker run --rm --network hermes-network \
		-v $$(CURDIR)/$(call MIGRATION_DIR,$(1)):/migrations:ro \
		-e DATABASE_URL=$(call DB_URL_NET,$(1)) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$$$f" && echo "Applied: $$$$f"; done'
	@echo -e "$$(GREEN)Done$$(NC)"
endef
$(foreach s,$(DB_SERVICES),$(eval $(call MIGRATE_RULE,$(s))))

##@ Database — Seeds

.PHONY: db-seed db-reset
db-seed: $(addprefix db-seed-,$(DB_SERVICES)) ## Run all seeds via Docker network
	@echo -e "$(GREEN)All seeds completed$(NC)"

# db-seed-<svc>: skips silently when no .sql files exist
define SEED_RULE
.PHONY: db-seed-$(1)
db-seed-$(1): ## Run $(1)-service seeds via Docker network
	@echo -e "$$(BLUE)Seeding hermes_$(1)...$$(NC)"
	@if [ -d "$(call SEED_DIR,$(1))" ] && ls $(call SEED_DIR,$(1))/*.sql 1>/dev/null 2>&1; then \
		docker run --rm --network hermes-network \
			-v $$(CURDIR)/$(call SEED_DIR,$(1)):/seeds:ro \
			postgres:16-alpine \
			sh -c 'for f in /seeds/*.sql; do psql "$(call DB_URL_NET,$(1))" -v ON_ERROR_STOP=1 -f "$$$$f" && echo "Seeded: $$$$f"; done'; \
	fi
	@echo -e "$$(GREEN)Done$$(NC)"
endef
$(foreach s,$(DB_SERVICES),$(eval $(call SEED_RULE,$(s))))

db-reset: clean up ## Clean, restart infra (services apply migrations on startup)
	@echo -e "$(GREEN)Database reset completed — start services to apply migrations$(NC)"

##@ Database — Shells

define DB_SHELL_RULE
.PHONY: db-shell-$(1)
db-shell-$(1): ## Open psql shell for hermes_$(1)
	@docker exec -it hermes-postgres psql -U hermes -d hermes_$(1)
endef
$(foreach s,$(DB_SERVICES),$(eval $(call DB_SHELL_RULE,$(s))))

##@ SQLx Offline Metadata

.PHONY: sqlx-prepare
sqlx-prepare: $(addprefix sqlx-prepare-,$(DB_SERVICES)) ## Generate SQLx offline metadata for all services
	@echo -e "$(GREEN)Done. Commit the .sqlx/ directories.$(NC)"

define SQLX_RULE
.PHONY: sqlx-prepare-$(1)
sqlx-prepare-$(1): ## Generate SQLx offline metadata for $(1)-service
	@echo -e "$$(BLUE)Preparing SQLx metadata for $(1)-service...$$(NC)"
	@DATABASE_URL=$(call DB_URL,$(1)) cargo sqlx prepare -p $(1)-service
	@echo -e "$$(GREEN)Done$$(NC)"
endef
$(foreach s,$(DB_SERVICES),$(eval $(call SQLX_RULE,$(s))))

##@ Development — Local (cargo run)

.PHONY: dev dev-all
dev: up ## Start infra (migrations run automatically on service startup)
	@echo -e "$(GREEN)Infrastructure ready!$(NC)"
	@echo -e "$(YELLOW)Start services: make run-{service}  — migrations apply on first startup$(NC)"
	@echo -e "$(YELLOW)Seed data:      make db-seed        — run after services are up$(NC)"

# run-<svc> for services with a database
define RUN_DB_RULE
.PHONY: run-$(1)
run-$(1): ## Run $(1)-service locally
	@echo -e "$$(BLUE)Starting $(1)-service...$$(NC)"
	@DATABASE_URL=$(call DB_URL,$(1)) cargo run --bin $(1)-service
endef
$(foreach s,$(RUN_DB_SERVICES),$(eval $(call RUN_DB_RULE,$(s))))

# run-<svc> for services without a database
define RUN_PLAIN_RULE
.PHONY: run-$(1)
run-$(1): ## Run $(1)-service locally
	@echo -e "$$(BLUE)Starting $(1)-service...$$(NC)"
	@cargo run --bin $(1)-service
endef
$(foreach s,$(RUN_PLAIN_SERVICES),$(eval $(call RUN_PLAIN_RULE,$(s))))

dev-all: ## Print commands to start all services in separate terminals
	@echo -e "$(YELLOW)Run these commands in separate terminals:$(NC)"
	@for s in $(RUN_DB_SERVICES) chat realtime; do echo "  make run-$$s"; done

##@ Building & Testing

.PHONY: build build-release test test-verbose test-watch check
build: ## Build all services
	@echo -e "$(BLUE)Building all services...$(NC)"
	@cargo build --workspace
	@echo -e "$(GREEN)Build completed$(NC)"

build-release: ## Build optimized release version
	@cargo build --workspace --release
	@echo -e "$(GREEN)Release build completed$(NC)"

test: ## Run all tests
	@echo -e "$(BLUE)Running tests...$(NC)"
	@cargo test --workspace
	@echo -e "$(GREEN)Tests passed$(NC)"

test-verbose: ## Run all tests with output
	@cargo test --workspace -- --nocapture

test-watch: ## Run tests in watch mode
	@cargo watch -x test

check: ## Check code without building
	@cargo check --workspace --all-targets
	@echo -e "$(GREEN)Check passed$(NC)"

##@ Testing — Integration
## ETC: make test-integration-auth or make test-integration-user
.PHONY: test-integration
test-integration: $(addprefix test-integration-,$(DB_SERVICES)) ## Run all integration tests
	@echo -e "$(GREEN)All integration tests completed$(NC)"

define INTEGRATION_RULE
.PHONY: test-integration-$(1)
test-integration-$(1): ## Run $(1)-service integration tests
	@echo -e "$$(BLUE)Running $(1)-service integration tests...$$(NC)"
	@cargo test -p $(1)-service --test $(1)_integration -- --test-threads=4
	@echo -e "$$(GREEN)Done$$(NC)"
endef
$(foreach s,$(DB_SERVICES),$(eval $(call INTEGRATION_RULE,$(s))))

##@ Code Quality

.PHONY: format fmt format-check lint fix audit
format: ## Format code with rustfmt
	@cargo fmt --all
	@echo -e "$(GREEN)Code formatted$(NC)"

fmt: format ## Alias for format

format-check: ## Check code formatting
	@cargo fmt --all -- --check

lint: ## Run clippy linter
	@cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo -e "$(GREEN)Lint passed$(NC)"

fix: ## Auto-fix linting issues
	@cargo clippy --fix --allow-dirty --allow-staged

audit: ## Run security audit
	@cargo audit
	@echo -e "$(GREEN)Audit completed$(NC)"

##@ Dependencies

.PHONY: update
update: ## Update dependencies
	@cargo update
	@echo -e "$(GREEN)Dependencies updated$(NC)"

##@ Monitoring & Tools

.PHONY: health grafana prometheus redis-cli nats-status
health: ## Check health status of all containers
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

grafana: ## Open Grafana dashboard
	@open http://localhost:3000 || xdg-open http://localhost:3000

prometheus: ## Open Prometheus dashboard
	@open http://localhost:9090 || xdg-open http://localhost:9090

redis-cli: ## Open Redis CLI
	@docker-compose -f $(INFRA_COMPOSE) exec redis redis-cli -a redis_dev_password

nats-status: ## Check NATS status
	@curl -s http://localhost:8222/varz | jq . || curl http://localhost:8222/varz

##@ Utility

.PHONY: ps shell-postgres shell-redis shell-nats
ps: ## Show running Docker containers
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

shell-postgres: ## Shell into PostgreSQL container
	@docker exec -it hermes-postgres sh

shell-redis: ## Shell into Redis container
	@docker exec -it hermes-redis sh

shell-nats: ## Shell into NATS container
	@docker exec -it hermes-nats sh

##@ API Testing (Schemathesis)

.PHONY: test-api test-api-auth test-api-user test-api-guild test-api-shell
test-api:       ## Run Schemathesis tests against all running services
	@bash scripts/test-api.sh all
test-api-auth:  ## Run Schemathesis tests against auth-service only
	@bash scripts/test-api.sh auth
test-api-user:  ## Run Schemathesis tests against user-service only
	@bash scripts/test-api.sh user
test-api-guild: ## Run Schemathesis tests against guild-service only
	@bash scripts/test-api.sh guild
test-api-shell: ## Open interactive Schemathesis shell
	@docker-compose -f $(INFRA_COMPOSE) --profile testing run --rm schemathesis sh

##@ CI/CD

.PHONY: ci pre-commit
ci: format-check lint test ## Run CI checks locally
	@echo -e "$(GREEN)All CI checks passed$(NC)"

pre-commit: format lint test ## Run before committing
	@echo -e "$(GREEN)Ready to commit$(NC)"

##@ Quick Actions

.PHONY: fresh quick-test status
fresh: clean dev ## Fresh start (clean + dev)
	@echo -e "$(GREEN)Fresh environment ready!$(NC)"

quick-test: test ## Quick test (no Docker restart)

status: ## Show project status
	@echo -e "$(BLUE)Docker Services:$(NC)"
	@docker ps --format "table {{.Names}}\t{{.Status}}"

##@ Advanced

.PHONY: tmux-dev kill-tmux watch-logs
tmux-dev: ## Start all services in tmux
	@tmux new-session -d -s hermes 'make run-realtime'
	@tmux split-window -h 'make run-auth'
	@tmux split-window -v 'make run-chat'
	@tmux select-pane -t 0
	@tmux split-window -v 'make run-user'
	@tmux attach-session -t hermes

kill-tmux: ## Kill tmux session
	@tmux kill-session -t hermes 2>/dev/null || echo "No tmux session found"

watch-logs: ## Watch infra logs with color highlighting
	@docker-compose -f $(INFRA_COMPOSE) logs -f | grep --color=auto -E 'ERROR|WARN|INFO|$$'
