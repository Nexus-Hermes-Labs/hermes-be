.PHONY: help build test clean run-infra run-all fmt lint docker-up docker-down

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'

build: ## Build all services
	cargo build --workspace --release

test: ## Run all tests
	cargo test --workspace

test-verbose: ## Run all tests with output
	cargo test --workspace -- --nocapture

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy lints
	cargo clippy --workspace --all-targets --all-features -- -D warnings

check: ## Check code without building
	cargo check --workspace

docker-up: ## Start all infrastructure services
	docker-compose up -d
	@echo "Waiting for services to be healthy..."
	@sleep 5
	docker-compose ps

docker-down: ## Stop all infrastructure services
	docker-compose down

docker-clean: ## Stop and remove all containers, volumes
	docker-compose down -v

run-infra: docker-up ## Alias for docker-up

run-gateway: ## Run gateway service
	cd crates/gateway-service && cargo run

run-auth: ## Run auth service
	cd crates/auth-service && cargo run

run-user: ## Run user service
	cd crates/user-service && cargo run

run-channel: ## Run channel service
	cd crates/channel-service && cargo run

run-chat: ## Run chat service
	cd crates/chat-service && cargo run

run-voice: ## Run voice service
	cd crates/voice-service && cargo run

run-stream: ## Run stream service
	cd crates/stream-service && cargo run

run-presence: ## Run presence service
	cd crates/presence-service && cargo run

run-media: ## Run media server
	cd crates/media-server && cargo run

logs: ## Show docker logs
	docker-compose logs -f

logs-postgres: ## Show postgres logs
	docker-compose logs -f postgres

logs-redis: ## Show redis logs
	docker-compose logs -f redis

logs-nats: ## Show nats logs
	docker-compose logs -f nats

db-shell: ## Open PostgreSQL shell
	docker-compose exec postgres psql -U hermes -d hermes

redis-cli: ## Open Redis CLI
	docker-compose exec redis redis-cli -a redis_dev_password

nats-status: ## Check NATS status
	curl http://localhost:8222/varz

setup: docker-up ## Initial setup
	@echo "Creating .env file from .env.example..."
	@cp -n .env.example .env || true
	@echo ""
	@echo "Setup complete! Edit .env file if needed."
	@echo "Run 'make build' to build all services."

dev-all: ## Run all services (requires tmux or separate terminals)
	@echo "Starting all services..."
	@echo "This requires tmux or you should run each service in separate terminal"
	@echo "Run: make run-gateway, make run-auth, etc. in different terminals"
