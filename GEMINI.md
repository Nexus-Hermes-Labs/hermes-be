# Hermes Project Overview

Hermes is a production-ready, Discord-like real-time communication platform built with Rust. It features a modular monolith architecture with service-oriented boundaries, supporting text chat, P2P voice communication, and presence tracking. The system is designed to evolve toward full microservices as scale demands.

## Architecture

Hermes is composed of 7 microservices, each with distinct responsibilities:

| Service      | Port | Responsibility                            |
|--------------|------|-------------------------------------------|
| **Gateway**  | 8080 | WebSocket gateway & REST API router       |
| **Auth**     | 8081 | User authentication & JWT management      |
| **User**     | 8082 | User profiles & friend system             |
| **Channel**  | 8083 | Server, channel & permission management   |
| **Chat**     | 8084 | Text messaging & reactions                |
| **Voice**    | 8085 | WebRTC P2P signaling                      |
| **Presence** | 8087 | Online status & typing indicators         |

## Technology Stack

**Backend:**
*   Rust 1.75+ with Axum web framework
*   PostgreSQL 16 for persistent storage (via `sqlx`)
*   Redis 7 for caching and pub/sub
*   NATS for event streaming
*   gRPC for inter-service communication

**Real-time:**
*   WebSocket for client connections
*   NATS for inter-service events
*   WebRTC for P2P voice calls

**Infrastructure:**
*   Docker & Docker Compose
*   Prometheus + Grafana for monitoring

## Getting Started

### Prerequisites
*   Rust 1.75+
*   Docker & Docker Compose

### Installation and Setup

1.  **Clone the repository:**
    ```bash
    git clone <your-repo-url>
    cd hermes
    ```
2.  **Copy environment variables:**
    ```bash
    cp .env.example .env
    ```
3.  **Start infrastructure (PostgreSQL, Redis, NATS, Prometheus, Grafana):**
    ```bash
    make up
    ```
4.  **Run database migrations:**
    ```bash
    make db-migrate
    ```
5.  **Seed the database (for development):**
    ```bash
    make db-seed
    ```
6.  **Build the project:**
    ```bash
    make build
    ```
7.  **Run individual services (in separate terminals or using tmux):**
    (Refer to `Makefile` for `run-*` commands, e.g., `make run-gateway`, `make run-auth`)

    Alternatively, to start all services using `tmux` (as defined in `Makefile`):
    ```bash
    make tmux-dev
    ```

### Running Tests

To run all tests across the workspace:
```bash
make test
```
For verbose output:
```bash
make test-verbose
```
To run tests for a specific service (e.g., `auth-service`):
```bash
cargo test -p auth-service
```

## Development Workflow

*   **Build all services:** `make build`
*   **Run linter:** `make lint`
*   **Format code:** `make format`
*   **Check code (without building):** `make check`
*   **Generate Protobuf code:** `make proto-generate`

## Monitoring

*   **Prometheus Dashboard:** Access at `http://localhost:9090`
*   **Grafana Dashboard:** Access at `http://localhost:3000` (admin/admin)

## Key Directories

*   `services/`: Contains all microservices and shared `common` library.
    *   `services/common/`: Shared models, errors, utilities, database migrations, and seed data.
*   `proto/`: Protocol buffer definitions for gRPC.
*   `docs/`: Project documentation (Architecture, Roadmap).
*   `infra/`: Infrastructure configuration files (Postgres, Prometheus, Grafana).
*   `.github/workflows/`: CI/CD GitHub Actions workflows.
*   `docker-compose.yml`: Defines the Docker infrastructure services.
*   `Makefile`: Provides common commands for development, testing, and operations.
*   `Cargo.toml`: Workspace configuration and dependency management.
