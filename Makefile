.PHONY: help up down restart clean logs db-migrate db-migrate-auth db-migrate-user db-migrate-guild db-migrate-channel db-migrate-messaging db-seed db-seed-auth db-seed-user db-seed-guild db-seed-channel db-seed-messaging db-reset dev test build format lint check install test-api test-api-auth test-api-user test-api-guild test-api-shell sqlx-prepare run-messaging run-chat

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Variables
COMPOSE := docker-compose
AUTH_MIGRATION_PATH := services/auth-service/migrations
USER_MIGRATION_PATH := services/user-service/migrations
GUILD_MIGRATION_PATH := services/guild-service/migrations
CHANNEL_MIGRATION_PATH := services/channel-service/migrations
MESSAGING_MIGRATION_PATH := services/messaging-service/migrations
AUTH_SEED_PATH := services/auth-service/seeds/dev
USER_SEED_PATH := services/user-service/seeds/dev
GUILD_SEED_PATH := services/guild-service/seeds/dev
CHANNEL_SEED_PATH := services/channel-service/seeds/dev
MESSAGING_SEED_PATH := services/messaging-service/seeds/dev
DB_URL := postgres://hermes:hermes@localhost:5432/hermes

##@ Help

help: ## Display this help message
	@echo -e "$(BLUE)Hermes - Microservices Platform$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make $(GREEN)<target>$(NC)\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Setup

install: ## Install development dependencies
	@echo -e "$(BLUE)📦 Installing dependencies...$(NC)"
	@cargo install sqlx-cli --no-default-features --features postgres
	@cargo install cargo-watch
	@cargo install cargo-audit
	@echo -e "$(GREEN)✅ Dependencies installed$(NC)"

setup: docker-up ## Initial project setup
	@echo -e "$(BLUE)⚙️  Setting up project...$(NC)"
	@cp -n .env.example .env || true
	@make db-migrate
	@make db-seed
	@echo ""
	@echo -e "$(GREEN)✅ Setup complete!$(NC)"
	@echo -e "$(YELLOW)Edit .env file if needed.$(NC)"
	@echo -e "$(YELLOW)Run 'make build' to build all services.$(NC)"

##@ Docker Management

up: ## Start all Docker services
	@echo -e "$(BLUE)🚀 Starting all services...$(NC)"
	@$(COMPOSE) up -d --wait
	@echo -e "$(YELLOW)⏳ Waiting for services to be healthy...$(NC)"
	@sleep 10
	@$(COMPOSE) ps
	@echo -e "$(GREEN)✅ Services started successfully$(NC)"

docker-up: up ## Alias for up

down: ## Stop all Docker services
	@echo -e "$(YELLOW)🛑 Stopping all services...$(NC)"
	@$(COMPOSE) down
	@echo -e "$(GREEN)✅ Services stopped$(NC)"

docker-down: down ## Alias for down

restart: down up ## Restart all Docker services

clean: ## Remove all containers, volumes, and networks
	@echo -e "$(RED)🗑️  Cleaning up Docker resources...$(NC)"
	@$(COMPOSE) down -v
	@docker rm -f $$(docker ps -aq) 2>/dev/null || true
	@docker volume rm $$(docker volume ls -q | grep hermes) 2>/dev/null || true
	@echo -e "$(GREEN)✅ Cleanup completed$(NC)"

docker-clean: clean ## Alias for clean

logs: ## Show logs from all services
	@$(COMPOSE) logs -f

logs-postgres: ## Show PostgreSQL logs
	@$(COMPOSE) logs -f postgres

logs-redis: ## Show Redis logs
	@$(COMPOSE) logs -f redis

logs-nats: ## Show NATS logs
	@$(COMPOSE) logs -f nats

logs-grafana: ## Show Grafana logs
	@$(COMPOSE) logs -f grafana

logs-prometheus: ## Show Prometheus logs
	@$(COMPOSE) logs -f prometheus

health: ## Check health status of all services
	@echo -e "$(BLUE)🏥 Checking service health...$(NC)"
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

##@ Protobuf / gRPC

proto-generate: ## Generate protobuf for all services
	@echo -e "$(BLUE)🧬 Generating all gRPC protobuf code...$(NC)"
	@cargo build -p auth-service
	@cargo build -p user-service
	@echo -e "$(GREEN)✅ All protobuf generated$(NC)"


proto-generate-auth:
	@echo -e "$(BLUE)🧬 Generating gRPC protobuf code (auth-service)...$(NC)"
	@cargo build -p auth-service
	@echo -e "$(GREEN)✅ Auth protobuf generated$(NC)"

proto-generate-user:
	@echo -e "$(BLUE)🧬 Generating gRPC protobuf code (user-service)...$(NC)"
	@cargo build -p user-service
	@echo -e "$(GREEN)✅ User protobuf generated$(NC)"

proto-clean: ## Clean all protobuf artifacts
	@echo -e "$(YELLOW)🧹 Cleaning all protobuf artifacts...$(NC)"
	@cargo clean -p auth-service
	@cargo clean -p user-service
	@echo -e "$(GREEN)✅ All cleaned$(NC)"


proto-clean-auth:
	@echo -e "$(YELLOW)🧹 Cleaning auth-service protobuf...$(NC)"
	@cargo clean -p auth-service
	@echo -e "$(GREEN)✅ Auth cleaned$(NC)"

proto-clean-user:
	@echo -e "$(YELLOW)🧹 Cleaning user-service protobuf...$(NC)"
	@cargo clean -p user-service
	@echo -e "$(GREEN)✅ User cleaned$(NC)"

##@ Database

sqlx-prepare: ## Generate SQLx offline metadata (run once before first Docker build)
	@echo -e "$(BLUE)🗄️  Preparing SQLx offline metadata...$(NC)"
	@cargo sqlx prepare --workspace
	@echo -e "$(GREEN)✅ Done. Commit the .sqlx/ directories.$(NC)"

db-migrate: db-migrate-auth db-migrate-user db-migrate-guild db-migrate-channel db-migrate-messaging ## Run all database migrations
	@echo -e "$(GREEN)✅ All migrations completed$(NC)"

db-migrate-auth: ## Run auth-service migrations
	@echo -e "$(BLUE)📦 Running auth-service migrations...$(NC)"
	@sqlx migrate run --source $(AUTH_MIGRATION_PATH) --ignore-missing
	@echo -e "$(GREEN)✅ Auth migrations completed$(NC)"

db-migrate-user: ## Run user-service migrations
	@echo -e "$(BLUE)📦 Running user-service migrations...$(NC)"
	@sqlx migrate run --source $(USER_MIGRATION_PATH) --ignore-missing
	@echo -e "$(GREEN)✅ User migrations completed$(NC)"

db-migrate-guild: ## Run guild-service migrations
	@echo -e "$(BLUE)📦 Running guild-service migrations...$(NC)"
	@sqlx migrate run --source $(GUILD_MIGRATION_PATH) --ignore-missing
	@echo -e "$(GREEN)✅ Guild migrations completed$(NC)"

db-migrate-channel: ## Run channel-service migrations
	@echo -e "$(BLUE)📦 Running channel-service migrations...$(NC)"
	@sqlx migrate run --source $(CHANNEL_MIGRATION_PATH) --ignore-missing
	@echo -e "$(GREEN)✅ Channel migrations completed$(NC)"

db-migrate-messaging: ## Run messaging-service migrations
	@echo -e "$(BLUE)📦 Running messaging-service migrations...$(NC)"
	@sqlx migrate run --source $(MESSAGING_MIGRATION_PATH) --ignore-missing
	@echo -e "$(GREEN)✅ Messaging migrations completed$(NC)"

db-seed: db-seed-auth db-seed-user db-seed-guild db-seed-channel db-seed-messaging ## Run all database seeds
	@echo -e "$(GREEN)✅ All seeds completed$(NC)"

db-seed-auth: ## Run auth-service seeds
	@echo -e "$(BLUE)🌱 Seeding auth database...$(NC)"
	@set -e; \
	for file in $$(ls $(AUTH_SEED_PATH)/*.sql | sort); do \
		echo "Running $$file"; \
		psql $(DB_URL) -v ON_ERROR_STOP=1 -f $$file; \
	done
	@echo -e "$(GREEN)✅ Auth database seeded$(NC)"

db-seed-user: ## Run user-service seeds
	@echo -e "$(BLUE)🌱 Seeding user database...$(NC)"
	@set -e; \
	for file in $$(ls $(USER_SEED_PATH)/*.sql | sort); do \
		echo "Running $$file"; \
		psql $(DB_URL) -v ON_ERROR_STOP=1 -f $$file; \
	done
	@echo -e "$(GREEN)✅ User database seeded$(NC)"

db-seed-guild: ## Run guild-service seeds
	@echo -e "$(BLUE)🌱 Seeding guild database...$(NC)"
	@set -e; \
	for file in $$(ls $(GUILD_SEED_PATH)/*.sql | sort); do \
		echo "Running $$file"; \
		psql $(DB_URL) -v ON_ERROR_STOP=1 -f $$file; \
	done
	@echo -e "$(GREEN)✅ Guild database seeded$(NC)"

db-seed-channel: ## Run channel-service seeds
	@echo -e "$(BLUE)🌱 Seeding channel database...$(NC)"
	@set -e; \
	if [ -d "$(CHANNEL_SEED_PATH)" ]; then \
		for file in $$(ls $(CHANNEL_SEED_PATH)/*.sql | sort); do \
			echo "Running $$file"; \
			psql $(DB_URL) -v ON_ERROR_STOP=1 -f $$file; \
		done; \
	fi
	@echo -e "$(GREEN)✅ Channel database seeded$(NC)"

db-seed-messaging: ## Run messaging-service seeds
	@echo -e "$(BLUE)🌱 Seeding messaging database...$(NC)"
	@set -e; \
	if [ -d "$(MESSAGING_SEED_PATH)" ]; then \
		for file in $$(ls $(MESSAGING_SEED_PATH)/*.sql | sort); do \
			echo "Running $$file"; \
			psql $(DB_URL) -v ON_ERROR_STOP=1 -f $$file; \
		done; \
	fi
	@echo -e "$(GREEN)✅ Messaging database seeded$(NC)"

db-reset: clean up db-migrate db-seed ## Clean, start, migrate, and seed database
	@echo -e "$(GREEN)✅ Database reset completed$(NC)"

db-shell: ## Open PostgreSQL shell
	@$(COMPOSE) exec postgres psql -U hermes -d hermes

db-create: ## Create database (if not exists)
	@echo -e "$(BLUE)📦 Creating database...$(NC)"
	@sqlx database create
	@echo -e "$(GREEN)✅ Database created$(NC)"

db-drop: ## Drop database
	@echo -e "$(RED)⚠️  Dropping database...$(NC)"
	@sqlx database drop -y
	@echo -e "$(GREEN)✅ Database dropped$(NC)"

##@ Development

dev: ## Start development environment
	@make up
	@sleep 3
	@make db-migrate
	@make db-seed
	@echo -e "$(GREEN)🎯 Development environment ready!$(NC)"
	@echo -e "$(YELLOW)Run services with: make run-auth, make run-user, etc.$(NC)"

run-infra: up ## Start infrastructure services (alias)

run-realtime: ## Run realtime service
	@echo -e "$(BLUE)🚀 Starting Realtime Service...$(NC)"
	@cargo run --bin realtime-service

run-auth: ## Run auth service
	@echo -e "$(BLUE)🚀 Starting Auth Service...$(NC)"
	@cargo run --bin auth-service

run-user: ## Run user service
	@echo -e "$(BLUE)🚀 Starting User Service...$(NC)"
	@cargo run --bin user-service

run-channel: ## Run channel service
	@echo -e "$(BLUE)🚀 Starting Channel Service...$(NC)"
	@cargo run --bin channel-service

run-chat: ## Run chat service
	@echo -e "$(BLUE)🚀 Starting Chat Service...$(NC)"
	@cargo run --bin chat-service

run-voice: ## Run voice service
	@echo -e "$(BLUE)🚀 Starting Voice Service...$(NC)"
	@cargo run --bin voice-service

run-presence: ## Run presence service
	@echo -e "$(BLUE)🚀 Starting Presence Service...$(NC)"
	@cargo run --bin presence-service

run-guild: ## Run guild service
	@echo -e "$(BLUE)🚀 Starting Guild Service...$(NC)"
	@cargo run --bin guild-service

run-messaging: ## Run messaging service
	@echo -e "$(BLUE)🚀 Starting Messaging Service...$(NC)"
	@cargo run --bin messaging-service

run-media: ## Run media service
	@echo -e "$(BLUE)🚀 Starting Media Service...$(NC)"
	@cargo run --bin media-service

run-notification: ## Run notification service
	@echo -e "$(BLUE)🚀 Starting Notification Service...$(NC)"
	@cargo run --bin notification-service

run-search: ## Run search service
	@echo -e "$(BLUE)🚀 Starting Search Service...$(NC)"
	@cargo run --bin search-service

run-ai: ## Run AI service
	@echo -e "$(BLUE)🚀 Starting AI Service...$(NC)"
	@cargo run --bin ai-service

dev-all: ## Run all services (requires tmux or separate terminals)
	@echo -e "$(YELLOW)⚠️  Starting all services requires multiple terminals$(NC)"
	@echo ""
	@echo -e "$(BLUE)Run these commands in separate terminals:$(NC)"
	@echo "  make run-auth"
	@echo "  make run-user"
	@echo "  make run-guild"
	@echo "  make run-channel"
	@echo "  make run-chat"
	@echo "  make run-voice"
	@echo "  make run-presence"
	@echo "  make run-realtime"
	@echo "  make run-media"
	@echo "  make run-notification"
	@echo "  make run-search"
	@echo "  make run-ai"

##@ Building & Testing

build: ## Build all services
	@echo -e "$(BLUE)🔨 Building all services...$(NC)"
	@cargo build --workspace
	@echo -e "$(GREEN)✅ Build completed$(NC)"

build-release: ## Build optimized release version
	@echo -e "$(BLUE)🔨 Building release...$(NC)"
	@cargo build --workspace --release
	@echo -e "$(GREEN)✅ Release build completed$(NC)"

test: ## Run all tests
	@echo -e "$(BLUE)🧪 Running tests...$(NC)"
	@cargo test --workspace
	@echo -e "$(GREEN)✅ Tests passed$(NC)"

test-verbose: ## Run all tests with output
	@echo -e "$(BLUE)🧪 Running tests with output...$(NC)"
	@cargo test --workspace -- --nocapture

test-watch: ## Run tests in watch mode
	@cargo watch -x test

check: ## Check code without building
	@echo -e "$(BLUE)🔍 Running cargo check...$(NC)"
	@cargo check --workspace --all-targets
	@echo -e "$(GREEN)✅ Check passed$(NC)"

##@ Code Quality

format: ## Format code with rustfmt
	@echo -e "$(BLUE)✨ Formatting code...$(NC)"
	@cargo fmt --all
	@echo -e "$(GREEN)✅ Code formatted$(NC)"

fmt: format ## Alias for format

format-check: ## Check code formatting
	@cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo -e "$(BLUE)🔍 Running clippy...$(NC)"
	@cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo -e "$(GREEN)✅ Lint passed$(NC)"

fix: ## Auto-fix linting issues
	@cargo clippy --fix --allow-dirty --allow-staged

audit: ## Run security audit
	@echo -e "$(BLUE)🔒 Running security audit...$(NC)"
	@cargo audit
	@echo -e "$(GREEN)✅ Audit completed$(NC)"

##@ Dependencies

update: ## Update dependencies
	@echo -e "$(BLUE)📦 Updating dependencies...$(NC)"
	@cargo update
	@echo -e "$(GREEN)✅ Dependencies updated$(NC)"

##@ Monitoring & Tools

grafana: ## Open Grafana dashboard
	@open http://localhost:3000 || xdg-open http://localhost:3000

prometheus: ## Open Prometheus dashboard
	@open http://localhost:9090 || xdg-open http://localhost:9090

redis-cli: ## Open Redis CLI
	@$(COMPOSE) exec redis redis-cli -a redis_dev_password

nats-status: ## Check NATS status
	@echo -e "$(BLUE)📊 NATS Server Status:$(NC)"
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

test-api: ## Run Schemathesis tests against all running services (auth + user + guild)
	@echo -e "$(BLUE)🔬 Running Schemathesis API tests (all services)...$(NC)"
	@bash scripts/test-api.sh all

test-api-auth: ## Run Schemathesis tests against auth-service only
	@echo -e "$(BLUE)🔬 Running Schemathesis API tests (auth-service)...$(NC)"
	@bash scripts/test-api.sh auth

test-api-user: ## Run Schemathesis tests against user-service only
	@echo -e "$(BLUE)🔬 Running Schemathesis API tests (user-service)...$(NC)"
	@bash scripts/test-api.sh user

test-api-guild: ## Run Schemathesis tests against guild-service only
	@echo -e "$(BLUE)🔬 Running Schemathesis API tests (guild-service)...$(NC)"
	@bash scripts/test-api.sh guild

test-api-shell: ## Open interactive Schemathesis shell (docker-compose)
	@$(COMPOSE) --profile testing run --rm schemathesis sh

##@ CI/CD

ci: format-check lint test ## Run CI checks locally
	@echo -e "$(GREEN)✅ All CI checks passed$(NC)"

pre-commit: format lint test ## Run before committing
	@echo -e "$(GREEN)✅ Ready to commit$(NC)"

##@ Quick Actions

fresh: clean dev ## Fresh start (clean + dev environment)
	@echo -e "$(GREEN)🎉 Fresh environment ready!$(NC)"

quick-test: ## Quick test (no Docker restart)
	@make db-migrate
	@make db-seed
	@make test

status: ## Show project status
	@echo -e "$(BLUE)📊 Project Status$(NC)"
	@echo ""
	@echo -e "$(YELLOW)Docker Services:$(NC)"
	@docker ps --format "table {{.Names}}\t{{.Status}}"
	@echo ""
	@echo -e "$(YELLOW)Database:$(NC)"
	@psql $(DB_URL) -c "SELECT COUNT(*) as user_count FROM users;" 2>/dev/null || echo "Database not accessible"
	@echo ""
	@echo -e "$(YELLOW)NATS:$(NC)"
	@curl -s http://localhost:8222/varz | jq -r '.server_id, .version' 2>/dev/null || echo "NATS not accessible"

##@ Advanced

tmux-dev: ## Start all services in tmux
	@echo -e "$(BLUE)🚀 Starting all services in tmux...$(NC)"
	@tmux new-session -d -s hermes 'make run-realtime'
	@tmux split-window -h 'make run-auth'
	@tmux split-window -v 'make run-chat'
	@tmux select-pane -t 0
	@tmux split-window -v 'make run-user'
	@tmux attach-session -t hermes

kill-tmux: ## Kill tmux session
	@tmux kill-session -t hermes 2>/dev/null || echo "No tmux session found"

watch-logs: ## Watch logs with color highlighting
	@$(COMPOSE) logs -f | grep --color=auto -E 'ERROR|WARN|INFO|$$'
