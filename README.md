# Hermes 🚀

A production-ready, Discord-like real-time communication platform built with
Rust. Features text chat, P2P voice communication, and presence tracking using a
scalable microservice architecture.

## ✨ Features

### Real-time Communication

- **Text Messaging**: Instant messaging in channels and direct messages with
  reactions and mentions
- **Voice Calls**: P2P voice communication using WebRTC (supports 2 users per
  channel)
- **Presence Tracking**: Real-time online/offline/idle/dnd status and typing
  indicators
- **Event-Driven**: Instant updates across all connected clients using NATS

### Server Management

- **Servers & Channels**: Create and manage Discord-like servers with text and
  voice channels
- **Roles & Permissions**: Bitflag-based permission system with role hierarchy
- **Member Management**: Invite users, manage members, and moderation tools

### User Features

- **Authentication**: Secure JWT-based auth with Argon2id password hashing
- **User Profiles**: Customizable profiles with avatars and bios
- **Friend System**: Add friends, accept requests, and manage relationships

## 🏗️ Architecture

### 7 Microservices

| Service      | Port | Responsibility                          |
| ------------ | ---- | --------------------------------------- |
| **Gateway**  | 8080 | WebSocket gateway & REST API router     |
| **Auth**     | 8081 | User authentication & JWT management    |
| **User**     | 8082 | User profiles & friend system           |
| **Channel**  | 8083 | Server, channel & permission management |
| **Chat**     | 8084 | Text messaging & reactions              |
| **Voice**    | 8085 | WebRTC P2P signaling                    |
| **Presence** | 8087 | Online status & typing indicators       |

### Technology Stack

**Backend:**

- Rust 1.75+ with Axum web framework
- PostgreSQL 16 for persistent storage
- Redis 7 for caching and pub/sub
- NATS for event streaming
- SQLx for compile-time checked queries

**Real-time:**

- WebSocket for client connections
- NATS for inter-service events
- WebRTC for P2P voice calls

**Infrastructure:**

- Docker & Docker Compose
- Prometheus + Grafana monitoring

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75 or higher ([install](https://rustup.rs/))
- **Docker** & Docker Compose
- **4GB RAM** minimum

### Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd hermes-clone

# Copy environment variables
cp .env.example .env

# Start infrastructure (PostgreSQL, Redis, NATS)
docker-compose up -d

# Run database migrations
sqlx migrate run --source crates/common/migrations

# Build the project
cargo build --workspace

# Start all services (in separate terminals or use tmux)
cargo run -p gateway-service    # Terminal 1: http://localhost:8080
cargo run -p auth-service       # Terminal 2: http://localhost:8081
cargo run -p user-service       # Terminal 3: http://localhost:8082
cargo run -p channel-service    # Terminal 4: http://localhost:8083
cargo run -p chat-service       # Terminal 5: http://localhost:8084
cargo run -p voice-service      # Terminal 6: http://localhost:8085
cargo run -p presence-service   # Terminal 7: http://localhost:8087
```

### Verify Installation

```bash
# Check all services are running
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health
# ... etc

# Run tests
cargo test --workspace
```

## 📖 Documentation

- **[Architecture Guide](docs/ARCHITECTURE.md)** - Detailed system design and
  service responsibilities
- **[Development Roadmap](docs/ROADMAP.md)** - 12-week development plan with
  weekly tasks
- **Service READMEs** - Each service has detailed documentation in
  `crates/*/README.md`

## 🎯 API Examples

### Authentication

```bash
# Register a new user
curl -X POST http://localhost:8081/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'

# Login
curl -X POST http://localhost:8081/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'
```

### Server & Channel Management

```bash
# Create a server (requires JWT token)
curl -X POST http://localhost:8083/servers \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Server",
    "icon_url": "https://example.com/icon.jpg"
  }'

# Create a channel
curl -X POST http://localhost:8083/servers/<server-id>/channels \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "general",
    "type": "text"
  }'
```

### Messaging

```bash
# Send a message
curl -X POST http://localhost:8084/channels/<channel-id>/messages \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello, world!"
  }'

# Get message history
curl http://localhost:8084/channels/<channel-id>/messages \
  -H "Authorization: Bearer <your-token>"
```

### WebSocket Connection

```javascript
// Connect to WebSocket gateway
const ws = new WebSocket("ws://localhost:8080/ws?token=<your-jwt-token>");

// Listen for messages
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("Received:", data);
};

// Send a message
ws.send(
  JSON.stringify({
    op: 2,
    t: "MESSAGE_CREATE",
    d: {
      channel_id: "channel-uuid",
      content: "Hello from WebSocket!",
    },
  }),
);
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific service
cargo test -p auth-service

# Run with output
cargo test --workspace -- --nocapture

# Run linter
cargo clippy --workspace
```

## 📦 Project Structure

```
hermes-clone/
├── crates/
│   ├── common/              # Shared library (models, errors, utilities)
│   │   ├── migrations/      # Database migrations
│   │   └── seeds/           # Test data
│   ├── auth-service/        # Authentication
│   ├── user-service/        # User management
│   ├── channel-service/     # Server & channel management
│   ├── chat-service/        # Text messaging
│   ├── voice-service/       # P2P voice signaling
│   ├── presence-service/    # Online status
│   └── gateway-service/     # WebSocket gateway & API router
├── docs/                    # Documentation
│   ├── ARCHITECTURE.md      # System architecture
│   └── ROADMAP.md           # Development roadmap
├── infra/                   # Infrastructure configs
│   ├── postgres/            # Database init scripts
│   ├── prometheus/          # Metrics
│   └── grafana/             # Dashboards
├── docker-compose.yml       # Infrastructure services
├── Cargo.toml              # Workspace configuration
└── README.md               # This file
```

## 🎓 Learning Path

This project is designed to be completed in 12 weeks at 6 hours per week:

| Phase       | Weeks | Focus                                        |
| ----------- | ----- | -------------------------------------------- |
| **Phase 1** | 1-4   | Infrastructure, Auth, User, Channel services |
| **Phase 2** | 5-8   | Chat service, Gateway, WebSocket, Real-time  |
| **Phase 3** | 9-12  | Presence, P2P Voice, Testing, Polish         |

See [ROADMAP.md](docs/ROADMAP.md) for detailed weekly tasks.

## 🔧 Configuration

Configuration is managed through environment variables:

```bash
# Database
DATABASE_URL=postgres://hermes:hermes_dev_password@localhost:5432/hermes

# Redis
REDIS_URL=redis://:redis_dev_password@localhost:6379

# NATS
NATS_URL=nats://localhost:4222

# JWT
JWT_SECRET=your-secret-key-change-in-production

# Service Ports
AUTH_SERVICE_PORT=8081
USER_SERVICE_PORT=8082
CHANNEL_SERVICE_PORT=8083
CHAT_SERVICE_PORT=8084
VOICE_SERVICE_PORT=8085
PRESENCE_SERVICE_PORT=8087
GATEWAY_SERVICE_PORT=8080
```

## 📊 Monitoring

### Prometheus Metrics

Access Prometheus at: `http://localhost:9090`

Example queries:

```
up{job="gateway-service"}
http_requests_total
websocket_connections_active
messages_sent_total
```

### Grafana Dashboards

Access Grafana at: `http://localhost:3000` (admin/admin)

Pre-configured dashboards for:

- Service health
- Request latency
- Message throughput
- WebSocket connections

## 🤝 Contributing

Contributions are welcome! This is a learning project, so feel free to:

- Add new features
- Fix bugs
- Improve documentation
- Share your learnings

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Discord's architecture
- Built with the amazing Rust ecosystem
- Thanks to the open-source community

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [WebRTC Guide](https://webrtc.org/)
- [NATS Documentation](https://docs.nats.io/)

---

**Built with 🦀 Rust**

For questions or discussions, open an issue or check out the
[documentation](docs/).

Happy coding! 🚀
