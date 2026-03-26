.PHONY: help \
	up up-all up-auth up-user up-guild up-channel up-messaging up-chat up-realtime \
	down down-all down-auth down-user down-guild down-channel down-messaging down-chat down-realtime \
	restart restart-auth restart-user restart-guild restart-channel restart-messaging \
	clean logs \
	logs-postgres logs-redis logs-nats logs-grafana logs-prometheus \
	logs-auth logs-user logs-guild logs-channel logs-messaging logs-chat logs-realtime \
	db-migrate db-migrate-auth db-migrate-user db-migrate-guild db-migrate-channel db-migrate-messaging \
	db-seed db-seed-auth db-seed-user db-seed-guild db-seed-channel db-seed-messaging \
	db-reset db-shell-auth db-shell-user db-shell-guild db-shell-channel db-shell-messaging \
	dev test build format lint check install test-api test-api-auth test-api-user test-api-guild test-api-shell \
	sqlx-prepare sqlx-prepare-auth sqlx-prepare-user sqlx-prepare-guild sqlx-prepare-channel sqlx-prepare-messaging \
	run-auth run-user run-guild run-channel run-messaging run-chat run-realtime

# Colors for output
BLUE  := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED   := \033[0;31m
NC    := \033[0m

# Docker Compose files
INFRA_COMPOSE    := docker-compose.infra.yml
AUTH_COMPOSE     := docker-compose.auth.yml
USER_COMPOSE     := docker-compose.user.yml
GUILD_COMPOSE    := docker-compose.guild.yml
CHANNEL_COMPOSE  := docker-compose.channel.yml
MESSAGING_COMPOSE := docker-compose.messaging.yml
CHAT_COMPOSE     := docker-compose.chat.yml
REALTIME_COMPOSE := docker-compose.realtime.yml

# Migration paths
AUTH_MIGRATION_PATH      := services/auth-service/migrations
USER_MIGRATION_PATH      := services/user-service/migrations
GUILD_MIGRATION_PATH     := services/guild-service/migrations
CHANNEL_MIGRATION_PATH   := services/channel-service/migrations
MESSAGING_MIGRATION_PATH := services/messaging-service/migrations

# Seed paths
AUTH_SEED_PATH      := services/auth-service/seeds/dev
USER_SEED_PATH      := services/user-service/seeds/dev
GUILD_SEED_PATH     := services/guild-service/seeds/dev
CHANNEL_SEED_PATH   := services/channel-service/seeds/dev
MESSAGING_SEED_PATH := services/messaging-service/seeds/dev

# Per-service database URLs
DB_URL_AUTH      := postgres://hermes:hermes@localhost:5432/hermes_auth
DB_URL_USER      := postgres://hermes:hermes@localhost:5432/hermes_user
DB_URL_GUILD     := postgres://hermes:hermes@localhost:5432/hermes_guild
DB_URL_CHANNEL   := postgres://hermes:hermes@localhost:5432/hermes_channel
DB_URL_MESSAGING := postgres://hermes:hermes@localhost:5432/hermes_messaging

##@ Help

help: ## Display this help message
	@echo -e "$(BLUE)Hermes - Microservices Platform$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make $(GREEN)<target>$(NC)\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-28s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Setup

install: ## Install development dependencies
	@echo -e "$(BLUE)Installing dependencies...$(NC)"
	@cargo install sqlx-cli --no-default-features --features postgres
	@cargo install cargo-watch
	@cargo install cargo-audit
	@echo -e "$(GREEN)Done$(NC)"

setup: up ## Initial project setup (infra up + migrate + seed)
	@echo -e "$(BLUE)Setting up project...$(NC)"
	@cp -n .env.example .env || true
	@make db-migrate
	@make db-seed
	@echo -e "$(GREEN)Setup complete! Edit .env if needed.$(NC)"

##@ Docker — Infrastructure

up: ## Start infrastructure services (postgres, redis, nats, mailpit, monitoring)
	@echo -e "$(BLUE)Starting infrastructure...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) up -d --wait
	@sleep 3
	@echo -e "$(GREEN)Infrastructure ready$(NC)"

down: ## Stop infrastructure services
	@echo -e "$(YELLOW)Stopping infrastructure...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) down
	@echo -e "$(GREEN)Done$(NC)"

clean: ## Remove all containers, volumes, and networks
	@echo -e "$(RED)Cleaning up Docker resources...$(NC)"
	@docker-compose -f $(INFRA_COMPOSE) down -v
	@for f in $(AUTH_COMPOSE) $(USER_COMPOSE) $(GUILD_COMPOSE) $(CHANNEL_COMPOSE) $(MESSAGING_COMPOSE) $(CHAT_COMPOSE) $(REALTIME_COMPOSE); do \
		docker-compose -f $$f down -v 2>/dev/null || true; \
	done
	@docker volume rm $$(docker volume ls -q | grep hermes) 2>/dev/null || true
	@echo -e "$(GREEN)Cleanup completed$(NC)"

##@ Docker — Services

up-auth: ## Start auth-service container
	@echo -e "$(BLUE)Starting auth-service...$(NC)"
	@docker-compose -f $(AUTH_COMPOSE) up -d --wait
	@echo -e "$(GREEN)auth-service ready$(NC)"

up-user: ## Start user-service container
	@echo -e "$(BLUE)Starting user-service...$(NC)"
	@docker-compose -f $(USER_COMPOSE) up -d --wait
	@echo -e "$(GREEN)user-service ready$(NC)"

up-guild: ## Start guild-service container
	@echo -e "$(BLUE)Starting guild-service...$(NC)"
	@docker-compose -f $(GUILD_COMPOSE) up -d --wait
	@echo -e "$(GREEN)guild-service ready$(NC)"

up-channel: ## Start channel-service container
	@echo -e "$(BLUE)Starting channel-service...$(NC)"
	@docker-compose -f $(CHANNEL_COMPOSE) up -d --wait
	@echo -e "$(GREEN)channel-service ready$(NC)"

up-messaging: ## Start messaging-service container
	@echo -e "$(BLUE)Starting messaging-service...$(NC)"
	@docker-compose -f $(MESSAGING_COMPOSE) up -d --wait
	@echo -e "$(GREEN)messaging-service ready$(NC)"

up-chat: ## Start chat-service container
	@echo -e "$(BLUE)Starting chat-service...$(NC)"
	@docker-compose -f $(CHAT_COMPOSE) up -d --wait
	@echo -e "$(GREEN)chat-service ready$(NC)"

up-realtime: ## Start realtime-service container
	@echo -e "$(BLUE)Starting realtime-service...$(NC)"
	@docker-compose -f $(REALTIME_COMPOSE) up -d --wait
	@echo -e "$(GREEN)realtime-service ready$(NC)"

up-all: up up-auth up-user up-guild up-channel up-messaging up-chat up-realtime ## Start infra + all service containers

down-auth: ## Stop auth-service container
	@docker-compose -f $(AUTH_COMPOSE) down

down-user: ## Stop user-service container
	@docker-compose -f $(USER_COMPOSE) down

down-guild: ## Stop guild-service container
	@docker-compose -f $(GUILD_COMPOSE) down

down-channel: ## Stop channel-service container
	@docker-compose -f $(CHANNEL_COMPOSE) down

down-messaging: ## Stop messaging-service container
	@docker-compose -f $(MESSAGING_COMPOSE) down

down-chat: ## Stop chat-service container
	@docker-compose -f $(CHAT_COMPOSE) down

down-realtime: ## Stop realtime-service container
	@docker-compose -f $(REALTIME_COMPOSE) down

down-all: down-auth down-user down-guild down-channel down-messaging down-chat down-realtime down ## Stop all service containers + infra

restart-auth: down-auth up-auth ## Restart auth-service container
restart-user: down-user up-user ## Restart user-service container
restart-guild: down-guild up-guild ## Restart guild-service container
restart-channel: down-channel up-channel ## Restart channel-service container
restart-messaging: down-messaging up-messaging ## Restart messaging-service container

restart: down up ## Restart infrastructure

##@ Logs

logs: ## Show infra logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f

logs-postgres: ## Show PostgreSQL logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f postgres

logs-redis: ## Show Redis logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f redis

logs-nats: ## Show NATS logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f nats

logs-grafana: ## Show Grafana logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f grafana

logs-prometheus: ## Show Prometheus logs
	@docker-compose -f $(INFRA_COMPOSE) logs -f prometheus

logs-auth: ## Show auth-service logs
	@docker-compose -f $(AUTH_COMPOSE) logs -f auth-service

logs-user: ## Show user-service logs
	@docker-compose -f $(USER_COMPOSE) logs -f user-service

logs-guild: ## Show guild-service logs
	@docker-compose -f $(GUILD_COMPOSE) logs -f guild-service

logs-channel: ## Show channel-service logs
	@docker-compose -f $(CHANNEL_COMPOSE) logs -f channel-service

logs-messaging: ## Show messaging-service logs
	@docker-compose -f $(MESSAGING_COMPOSE) logs -f messaging-service

logs-chat: ## Show chat-service logs
	@docker-compose -f $(CHAT_COMPOSE) logs -f chat-service

logs-realtime: ## Show realtime-service logs
	@docker-compose -f $(REALTIME_COMPOSE) logs -f realtime-service

##@ Protobuf / gRPC

proto-generate: ## Generate protobuf for all services
	@echo -e "$(BLUE)Generating all gRPC protobuf code...$(NC)"
	@cargo build -p auth-service
	@cargo build -p user-service
	@echo -e "$(GREEN)Done$(NC)"

proto-generate-auth:
	@cargo build -p auth-service

proto-generate-user:
	@cargo build -p user-service

proto-clean: ## Clean all protobuf artifacts
	@cargo clean -p auth-service
	@cargo clean -p user-service

##@ Database — Migrations
# Migrations are embedded via sqlx::migrate!() and run automatically on service startup.
# These targets are kept for CI/CD environments with direct postgres access.

# Docker-network database URLs (used from within hermes-network)
DB_URL_AUTH_NET      := postgres://hermes:hermes@hermes-postgres:5432/hermes_auth
DB_URL_USER_NET      := postgres://hermes:hermes@hermes-postgres:5432/hermes_user
DB_URL_GUILD_NET     := postgres://hermes:hermes@hermes-postgres:5432/hermes_guild
DB_URL_CHANNEL_NET   := postgres://hermes:hermes@hermes-postgres:5432/hermes_channel
DB_URL_MESSAGING_NET := postgres://hermes:hermes@hermes-postgres:5432/hermes_messaging

db-migrate: ## Migrations run automatically on service startup (see db-migrate-{service} for manual runs)
	@echo -e "$(YELLOW)Migrations run automatically when services start.$(NC)"
	@echo -e "$(YELLOW)Use 'make up-all' to start services and apply migrations.$(NC)"

db-migrate-auth: ## Run auth-service migrations via Docker network
	@echo -e "$(BLUE)Running auth-service migrations...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(AUTH_MIGRATION_PATH):/migrations:ro \
		-e DATABASE_URL=$(DB_URL_AUTH_NET) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$f" && echo "Applied: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-migrate-user: ## Run user-service migrations via Docker network
	@echo -e "$(BLUE)Running user-service migrations...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(USER_MIGRATION_PATH):/migrations:ro \
		-e DATABASE_URL=$(DB_URL_USER_NET) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$f" && echo "Applied: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-migrate-guild: ## Run guild-service migrations via Docker network
	@echo -e "$(BLUE)Running guild-service migrations...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(GUILD_MIGRATION_PATH):/migrations:ro \
		-e DATABASE_URL=$(DB_URL_GUILD_NET) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$f" && echo "Applied: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-migrate-channel: ## Run channel-service migrations via Docker network
	@echo -e "$(BLUE)Running channel-service migrations...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(CHANNEL_MIGRATION_PATH):/migrations:ro \
		-e DATABASE_URL=$(DB_URL_CHANNEL_NET) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$f" && echo "Applied: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-migrate-messaging: ## Run messaging-service migrations via Docker network
	@echo -e "$(BLUE)Running messaging-service migrations...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(MESSAGING_MIGRATION_PATH):/migrations:ro \
		-e DATABASE_URL=$(DB_URL_MESSAGING_NET) \
		postgres:16-alpine \
		sh -c 'for f in /migrations/*.sql; do psql "$$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$$f" && echo "Applied: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

##@ Database — Seeds

db-seed: db-seed-auth db-seed-user db-seed-guild db-seed-channel db-seed-messaging ## Run all seeds via Docker network
	@echo -e "$(GREEN)All seeds completed$(NC)"

db-seed-auth: ## Run auth-service seeds via Docker network
	@echo -e "$(BLUE)Seeding hermes_auth...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(AUTH_SEED_PATH):/seeds:ro \
		postgres:16-alpine \
		sh -c 'for f in /seeds/*.sql; do psql "$(DB_URL_AUTH_NET)" -v ON_ERROR_STOP=1 -f "$$f" && echo "Seeded: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-seed-user: ## Run user-service seeds via Docker network
	@echo -e "$(BLUE)Seeding hermes_user...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(USER_SEED_PATH):/seeds:ro \
		postgres:16-alpine \
		sh -c 'for f in /seeds/*.sql; do psql "$(DB_URL_USER_NET)" -v ON_ERROR_STOP=1 -f "$$f" && echo "Seeded: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-seed-guild: ## Run guild-service seeds via Docker network
	@echo -e "$(BLUE)Seeding hermes_guild...$(NC)"
	@docker run --rm --network hermes-network \
		-v $(CURDIR)/$(GUILD_SEED_PATH):/seeds:ro \
		postgres:16-alpine \
		sh -c 'for f in /seeds/*.sql; do psql "$(DB_URL_GUILD_NET)" -v ON_ERROR_STOP=1 -f "$$f" && echo "Seeded: $$f"; done'
	@echo -e "$(GREEN)Done$(NC)"

db-seed-channel: ## Run channel-service seeds via Docker network
	@echo -e "$(BLUE)Seeding hermes_channel...$(NC)"
	@if [ -d "$(CHANNEL_SEED_PATH)" ] && ls $(CHANNEL_SEED_PATH)/*.sql 1>/dev/null 2>&1; then \
		docker run --rm --network hermes-network \
			-v $(CURDIR)/$(CHANNEL_SEED_PATH):/seeds:ro \
			postgres:16-alpine \
			sh -c 'for f in /seeds/*.sql; do psql "$(DB_URL_CHANNEL_NET)" -v ON_ERROR_STOP=1 -f "$$f" && echo "Seeded: $$f"; done'; \
	fi
	@echo -e "$(GREEN)Done$(NC)"

db-seed-messaging: ## Run messaging-service seeds via Docker network
	@echo -e "$(BLUE)Seeding hermes_messaging...$(NC)"
	@if [ -d "$(MESSAGING_SEED_PATH)" ] && ls $(MESSAGING_SEED_PATH)/*.sql 1>/dev/null 2>&1; then \
		docker run --rm --network hermes-network \
			-v $(CURDIR)/$(MESSAGING_SEED_PATH):/seeds:ro \
			postgres:16-alpine \
			sh -c 'for f in /seeds/*.sql; do psql "$(DB_URL_MESSAGING_NET)" -v ON_ERROR_STOP=1 -f "$$f" && echo "Seeded: $$f"; done'; \
	fi
	@echo -e "$(GREEN)Done$(NC)"

db-reset: clean up ## Clean, restart infra (services apply migrations on startup)
	@echo -e "$(GREEN)Database reset completed — start services to apply migrations$(NC)"

##@ Database — Shells

db-shell-auth: ## Open psql shell for hermes_auth
	@docker exec -it hermes-postgres psql -U hermes -d hermes_auth

db-shell-user: ## Open psql shell for hermes_user
	@docker exec -it hermes-postgres psql -U hermes -d hermes_user

db-shell-guild: ## Open psql shell for hermes_guild
	@docker exec -it hermes-postgres psql -U hermes -d hermes_guild

db-shell-channel: ## Open psql shell for hermes_channel
	@docker exec -it hermes-postgres psql -U hermes -d hermes_channel

db-shell-messaging: ## Open psql shell for hermes_messaging
	@docker exec -it hermes-postgres psql -U hermes -d hermes_messaging

##@ SQLx Offline Metadata

sqlx-prepare: sqlx-prepare-auth sqlx-prepare-user sqlx-prepare-guild sqlx-prepare-channel sqlx-prepare-messaging ## Generate SQLx offline metadata for all services
	@echo -e "$(GREEN)Done. Commit the .sqlx/ directories.$(NC)"

sqlx-prepare-auth: ## Generate SQLx offline metadata for auth-service
	@echo -e "$(BLUE)Preparing SQLx metadata for auth-service...$(NC)"
	@DATABASE_URL=$(DB_URL_AUTH) cargo sqlx prepare -p auth-service
	@echo -e "$(GREEN)Done$(NC)"

sqlx-prepare-user: ## Generate SQLx offline metadata for user-service
	@echo -e "$(BLUE)Preparing SQLx metadata for user-service...$(NC)"
	@DATABASE_URL=$(DB_URL_USER) cargo sqlx prepare -p user-service
	@echo -e "$(GREEN)Done$(NC)"

sqlx-prepare-guild: ## Generate SQLx offline metadata for guild-service
	@echo -e "$(BLUE)Preparing SQLx metadata for guild-service...$(NC)"
	@DATABASE_URL=$(DB_URL_GUILD) cargo sqlx prepare -p guild-service
	@echo -e "$(GREEN)Done$(NC)"

sqlx-prepare-channel: ## Generate SQLx offline metadata for channel-service
	@echo -e "$(BLUE)Preparing SQLx metadata for channel-service...$(NC)"
	@DATABASE_URL=$(DB_URL_CHANNEL) cargo sqlx prepare -p channel-service
	@echo -e "$(GREEN)Done$(NC)"

sqlx-prepare-messaging: ## Generate SQLx offline metadata for messaging-service
	@echo -e "$(BLUE)Preparing SQLx metadata for messaging-service...$(NC)"
	@DATABASE_URL=$(DB_URL_MESSAGING) cargo sqlx prepare -p messaging-service
	@echo -e "$(GREEN)Done$(NC)"

##@ Development — Local (cargo run)

dev: up ## Start infra (migrations run automatically on service startup)
	@echo -e "$(GREEN)Infrastructure ready!$(NC)"
	@echo -e "$(YELLOW)Start services: make run-{service}  — migrations apply on first startup$(NC)"
	@echo -e "$(YELLOW)Seed data:      make db-seed        — run after services are up$(NC)"

run-auth: ## Run auth-service locally
	@echo -e "$(BLUE)Starting auth-service...$(NC)"
	@DATABASE_URL=$(DB_URL_AUTH) cargo run --bin auth-service

run-user: ## Run user-service locally
	@echo -e "$(BLUE)Starting user-service...$(NC)"
	@DATABASE_URL=$(DB_URL_USER) cargo run --bin user-service

run-guild: ## Run guild-service locally
	@echo -e "$(BLUE)Starting guild-service...$(NC)"
	@DATABASE_URL=$(DB_URL_GUILD) cargo run --bin guild-service

run-channel: ## Run channel-service locally
	@echo -e "$(BLUE)Starting channel-service...$(NC)"
	@DATABASE_URL=$(DB_URL_CHANNEL) cargo run --bin channel-service

run-messaging: ## Run messaging-service locally
	@echo -e "$(BLUE)Starting messaging-service...$(NC)"
	@DATABASE_URL=$(DB_URL_MESSAGING) cargo run --bin messaging-service

run-chat: ## Run chat-service locally
	@echo -e "$(BLUE)Starting chat-service...$(NC)"
	@cargo run --bin chat-service

run-realtime: ## Run realtime-service locally
	@echo -e "$(BLUE)Starting realtime-service...$(NC)"
	@cargo run --bin realtime-service

run-voice: ## Run voice-service locally
	@cargo run --bin voice-service

run-presence: ## Run presence-service locally
	@cargo run --bin presence-service

run-media: ## Run media-service locally
	@cargo run --bin media-service

run-notification: ## Run notification-service locally
	@cargo run --bin notification-service

run-search: ## Run search-service locally
	@cargo run --bin search-service

run-ai: ## Run AI service locally
	@cargo run --bin ai-service

dev-all: ## Print commands to start all services in separate terminals
	@echo -e "$(YELLOW)Run these commands in separate terminals:$(NC)"
	@echo "  make run-auth"
	@echo "  make run-user"
	@echo "  make run-guild"
	@echo "  make run-channel"
	@echo "  make run-messaging"
	@echo "  make run-chat"
	@echo "  make run-realtime"

##@ Building & Testing

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

##@ Code Quality

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

update: ## Update dependencies
	@cargo update
	@echo -e "$(GREEN)Dependencies updated$(NC)"

##@ Monitoring & Tools

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

ps: ## Show running Docker containers
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

shell-postgres: ## Shell into PostgreSQL container
	@docker exec -it hermes-postgres sh

shell-redis: ## Shell into Redis container
	@docker exec -it hermes-redis sh

shell-nats: ## Shell into NATS container
	@docker exec -it hermes-nats sh

##@ API Testing (Schemathesis)

test-api: ## Run Schemathesis tests against all running services
	@bash scripts/test-api.sh all

test-api-auth: ## Run Schemathesis tests against auth-service only
	@bash scripts/test-api.sh auth

test-api-user: ## Run Schemathesis tests against user-service only
	@bash scripts/test-api.sh user

test-api-guild: ## Run Schemathesis tests against guild-service only
	@bash scripts/test-api.sh guild

test-api-shell: ## Open interactive Schemathesis shell
	@docker-compose -f $(INFRA_COMPOSE) --profile testing run --rm schemathesis sh

##@ CI/CD

ci: format-check lint test ## Run CI checks locally
	@echo -e "$(GREEN)All CI checks passed$(NC)"

pre-commit: format lint test ## Run before committing
	@echo -e "$(GREEN)Ready to commit$(NC)"

##@ Quick Actions

fresh: clean dev ## Fresh start (clean + dev)
	@echo -e "$(GREEN)Fresh environment ready!$(NC)"

quick-test: test ## Quick test (no Docker restart)

status: ## Show project status
	@echo -e "$(BLUE)Docker Services:$(NC)"
	@docker ps --format "table {{.Names}}\t{{.Status}}"

##@ Advanced

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
