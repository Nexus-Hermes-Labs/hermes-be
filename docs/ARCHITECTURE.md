# Discord Clone - System Architecture

**Version:** 1.0.0-MVP  
**Last Updated:** Week 3, Day 2

## Table of Contents

- [Overview](#overview)
- [Architecture Principles](#architecture-principles)
- [System Architecture](#system-architecture)
- [Communication Patterns](#communication-patterns)
- [Data Architecture](#data-architecture)
- [Authentication & Authorization](#authentication--authorization)
- [Technology Stack](#technology-stack)
- [Deployment Architecture](#deployment-architecture)
- [Scalability Strategy](#scalability-strategy)
- [Monitoring & Observability](#monitoring--observability)

---

## Overview

Discord Clone is a microservices-based real-time communication platform built with Rust. The system follows Domain-Driven Design (DDD) principles with a hybrid communication architecture optimized for both synchronous and asynchronous operations.

### Architecture Goals

1. **Scalability**: Handle millions of concurrent users
2. **Low Latency**: <100ms p95 response time
3. **High Availability**: 99.9% uptime SLA
4. **Maintainability**: Clean separation of concerns
5. **Developer Experience**: Type-safe contracts, comprehensive testing

---

## Architecture Principles

### 1. Microservices Architecture

Each service owns its domain and can be deployed independently.
```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│Auth Service │  │User Service │  │Server       │
│             │  │             │  │Service      │
│ - Login     │  │ - Profile   │  │ - Servers   │
│ - Register  │  │ - Friends   │  │ - Channels  │
│ - JWT       │  │ - Blocks    │  │ - Members   │
└─────────────┘  └─────────────┘  └─────────────┘
```

**Benefits:**
- Independent scaling
- Technology diversity (all Rust for now, but flexible)
- Fault isolation
- Team autonomy

---

### 2. Domain-Driven Design (DDD)

Each service follows DDD layers:
```
┌──────────────────────────────────────┐
│      Presentation Layer              │ ← HTTP/gRPC handlers
├──────────────────────────────────────┤
│      Application Layer               │ ← Use cases, DTOs
├──────────────────────────────────────┤
│      Domain Layer                    │ ← Business logic, entities
├──────────────────────────────────────┤
│      Infrastructure Layer            │ ← Database, gRPC, cache
└──────────────────────────────────────┘
```

**Layer Responsibilities:**

- **Domain**: Pure business logic, no dependencies on infrastructure
- **Application**: Orchestrates domain logic, handles transactions
- **Infrastructure**: Implements repositories, external services
- **Presentation**: Handles protocol concerns (HTTP, gRPC)

---

### 3. Hybrid Communication

Different communication patterns for different needs:
```
┌─────────────────────────────────────────┐
│ SYNCHRONOUS (gRPC)                      │
│ - Must wait for response                │
│ - 1-to-1 communication                  │
│ - Strong consistency                    │
├─────────────────────────────────────────┤
│ ASYNCHRONOUS (NATS - Future)            │
│ - Fire-and-forget                       │
│ - 1-to-many communication               │
│ - Eventual consistency                  │
└─────────────────────────────────────────┘
```

---

## System Architecture

### High-Level Architecture (MVP)
```
┌──────────────────────────────────────────────────┐
│                                                  │
│            Client Applications                   │
│     (Web, iOS, Android, Desktop)                │
│                                                  │
└────────────────┬─────────────────────────────────┘
                 │ HTTPS/WebSocket
                 ▼
┌──────────────────────────────────────────────────┐
│           API Gateway (Future)                   │
│  - Load balancing                                │
│  - Rate limiting                                 │
│  - SSL termination                               │
└────────┬───────────────────┬─────────────────────┘
         │ HTTP              │ HTTP
         ▼                   ▼
┌─────────────────┐   ┌─────────────────┐
│  Auth Service   │   │  User Service   │
│   Port: 8081    │   │   Port: 8082    │
│                 │   │                 │
│ REST Endpoints  │   │ REST Endpoints  │
│                 │   │ gRPC Server     │
└────────┬────────┘   └────────┬────────┘
         │                     │
         │ gRPC (sync)         │
         └──────────┬──────────┘
                    │
                    ▼
         ┌──────────────────┐
         │   PostgreSQL     │
         │  (Shared - MVP)  │
         └──────────────────┘
```

### Post-MVP Architecture
```
┌──────────────────────────────────────────────────┐
│                Clients                           │
└────────────────┬─────────────────────────────────┘
                 │
                 ▼
         ┌──────────────┐
         │ API Gateway  │
         └──────┬───────┘
                │
      ┌─────────┼──────────┐
      │         │          │
      ▼         ▼          ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│  Auth    │ │  User    │ │  Server  │
│ Service  │ │ Service  │ │ Service  │
└────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │
     │ gRPC       │ gRPC       │ gRPC
     └────────────┼────────────┘
                  │
         ┌────────┴────────┐
         ▼                 ▼
   ┌──────────┐      ┌──────────┐
   │PostgreSQL│      │   NATS   │
   └──────────┘      │ JetStream│
                     └──────────┘
```

---

## Communication Patterns

### 1. Synchronous Communication (gRPC)

**Use Cases:**
- Service-to-service queries
- Operations requiring immediate response
- Data validation
- Permission checks

**Example Flow:**
```
Auth Service                User Service
     │                           │
     │ GenerateDiscriminator     │
     ├──────────────────────────►│
     │                           │
     │      discriminator        │
     │◄──────────────────────────┤
     │                           │
```

**Implementation:**
```rust
// Auth Service (Client)
let discriminator = user_client
    .generate_discriminator(GenerateDiscriminatorRequest {
        username: username.to_string(),
    })
    .await?
    .into_inner()
    .discriminator;

// User Service (Server)
async fn generate_discriminator(
    &self,
    request: Request<GenerateDiscriminatorRequest>,
) -> Result<Response<GenerateDiscriminatorResponse>, Status> {
    let discriminator = self.discriminator_service
        .generate_discriminator(&request.into_inner().username)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    
    Ok(Response::new(GenerateDiscriminatorResponse {
        discriminator,
    }))
}
```

**Benefits:**
- Type-safe contracts (Protocol Buffers)
- Low latency (~5-10ms on local network)
- HTTP/2 multiplexing
- Bidirectional streaming
- Auto-generated client code

---

### 2. Asynchronous Communication (NATS - Post-MVP)

**Use Cases:**
- Event notifications
- Cross-service updates
- Analytics tracking
- Audit logging

**Example Flow:**
```
User Service              NATS              Notification    Analytics
     │                     │                Service         Service
     │ UserCreatedEvent    │                    │              │
     ├────────────────────►│                    │              │
     │                     │ Distribute         │              │
     │                     ├───────────────────►│              │
     │                     ├──────────────────────────────────►│
     │                     │                    │              │
```

**Implementation:**
```rust
// Publisher (User Service)
self.event_bus.publish(UserCreatedEvent {
    event_id: Uuid::new_v4(),
    user_id: user.id,
    username: user.username,
    email: user.email,
    created_at: user.created_at,
}).await?;

// Subscriber (Notification Service)
let subscription = nats_client
    .subscribe("user_profile.created")
    .await?;

while let Some(msg) = subscription.next().await {
    let event: UserCreatedEvent = serde_json::from_slice(&msg.data)?;
    send_welcome_email(event).await?;
}
```

**Benefits:**
- Decoupled services
- Multiple subscribers
- Message persistence (JetStream)
- At-least-once delivery
- Automatic reconnection

---

### 3. Client Communication (HTTP REST)

**Use Cases:**
- Client-to-service API calls
- CRUD operations
- File uploads
- Public APIs

**Characteristics:**
- RESTful design
- JSON payloads
- JWT authentication
- Rate limiting
- CORS support

---

## Data Architecture

### Database Strategy

#### MVP: Shared Database
```
┌──────────────────────────────────────┐
│          PostgreSQL 16               │
├──────────────────────────────────────┤
│                                      │
│  ┌────────────┐    ┌──────────────┐ │
│  │   users    │    │ user_        │ │
│  │            │    │ relationships│ │
│  └────────────┘    └──────────────┘ │
│                                      │
│  Auth & User Services share DB      │
│  (Simplicity for MVP)                │
└──────────────────────────────────────┘
```

**Rationale:**
- Simple for MVP
- No distributed transactions
- ACID guarantees
- Easy local development

**Trade-offs:**
- Tight coupling
- Single point of failure
- Scaling limitations

---

#### Post-MVP: Database per Service
```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ Auth DB     │  │ User DB     │  │ Server DB   │
│             │  │             │  │             │
│ - sessions  │  │ - users     │  │ - servers   │
│ - tokens    │  │ - friends   │  │ - channels  │
│             │  │ - blocks    │  │ - members   │
└─────────────┘  └─────────────┘  └─────────────┘
```

**Benefits:**
- Independent scaling
- Technology flexibility
- Fault isolation
- Clear ownership

**Challenges:**
- Distributed transactions (Saga pattern)
- Data consistency (eventual)
- Cross-service queries (API composition)

---

### Database Design Principles

#### 1. Single-Table Design for Bidirectional Relationships
```sql
-- User relationships (friends, blocks, pending)
CREATE TABLE user_relationships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,           -- Perspective owner
    target_user_id UUID NOT NULL,    -- Target user_profile
    type relationship_type NOT NULL,  -- friend, blocked, pending_*
    message TEXT,                     -- For friend requests
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    
    UNIQUE(user_id, target_user_id)
);

-- Bidirectional sync trigger
CREATE TRIGGER trg_sync_bidirectional_relationship
    AFTER INSERT OR UPDATE OR DELETE ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION sync_bidirectional_relationship();
```

**Benefits:**
- Automatic consistency (triggers)
- Single source of truth
- Efficient queries

---

#### 2. Indexing Strategy
```sql
-- Covering index for friends list
CREATE INDEX idx_relationships_friends 
ON user_relationships(user_id, created_at DESC) 
WHERE type = 'friend';

-- Covering index for pending incoming
CREATE INDEX idx_relationships_pending_incoming 
ON user_relationships(user_id, created_at DESC) 
WHERE type = 'pending_incoming';

-- Fast existence checks
CREATE INDEX idx_relationships_user_target 
ON user_relationships(user_id, target_user_id);
```

**Query Performance:**
- Friends list: O(log n) with index seek
- Existence check: O(1) hash lookup
- Count queries: Index-only scan

---

### Caching Strategy (Post-MVP)
```
┌──────────────────────────────────────┐
│            Redis Cluster             │
├──────────────────────────────────────┤
│                                      │
│  Cache Layers:                       │
│  ┌────────────────────────────────┐  │
│  │ L1: Session Cache (15min)     │  │
│  │ - JWT claims                   │  │
│  │ - User sessions                │  │
│  └────────────────────────────────┘  │
│                                      │
│  ┌────────────────────────────────┐  │
│  │ L2: User Profile Cache (1hr)  │  │
│  │ - User data                    │  │
│  │ - Friend counts                │  │
│  └────────────────────────────────┘  │
│                                      │
│  ┌────────────────────────────────┐  │
│  │ L3: Relationship Cache (5min) │  │
│  │ - are_friends(A, B) → bool    │  │
│  │ - is_blocked(A, B) → bool     │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

**Cache Invalidation:**
- Write-through: Update cache on DB write
- TTL-based: Auto-expire after duration
- Event-based: Invalidate on domain events

**Cache Keys Pattern:**
```
user:{user_id}:profile
user:{user_id}:friends:count
relationship:{user_id}:{target_id}:status
session:{session_id}
```

---

## Authentication & Authorization

### JWT-Based Authentication
```
┌──────────────────────────────────────────────────┐
│                 Auth Flow                        │
└──────────────────────────────────────────────────┘

1. Login Request
   ├─ POST /v1/auth/login
   └─ {email, password}

2. Auth Service validates credentials
   ├─ Hash password with Argon2
   └─ Compare with stored hash

3. Generate JWT tokens
   ├─ Access Token (15 min)
   └─ Refresh Token (7 days)

4. Return tokens to client
   └─ Client stores in secure storage

5. Subsequent Requests
   ├─ Authorization: Bearer <access_token>
   └─ Service validates JWT signature

6. Token Refresh
   ├─ POST /v1/auth/refresh
   ├─ {refresh_token}
   └─ New access + refresh tokens
```

### JWT Structure
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "550e8400-e29b-41d4-a716-446655440000",
    "username": "alice",
    "discriminator": "0042",
    "email": "alice@example.com",
    "exp": 1735689600,
    "iat": 1735603200,
    "jti": "unique-token-id"
  },
  "signature": "..."
}
```

### Authorization Patterns

#### 1. Resource Ownership
```rust
// Only user_profile can update their own profile
async fn update_profile(
    user_id: Uuid,           // From JWT
    profile_id: Uuid,        // From URL
    updates: UpdateProfile,
) -> Result<User> {
    if user_id != profile_id {
        return Err(Error::Forbidden);
    }
    // ... update logic
}
```

#### 2. Relationship-Based
```rust
// Only friends can see online status
async fn get_user_status(
    requester_id: Uuid,
    target_id: Uuid,
) -> Result<UserStatus> {
    let target = repo.find_by_id(target_id).await?;
    
    if !target.privacy.show_online_status {
        let are_friends = repo.are_friends(requester_id, target_id).await?;
        if !are_friends {
            return Ok(UserStatus::Offline); // Hide status
        }
    }
    
    Ok(target.status)
}
```

#### 3. Role-Based (Future)
```rust
// Server admin can delete channels
async fn delete_channel(
    user_id: Uuid,
    server_id: Uuid,
    channel_id: Uuid,
) -> Result<()> {
    let member = repo.get_server_member(user_id, server_id).await?;
    
    if !member.has_permission(Permission::ManageChannels) {
        return Err(Error::Forbidden);
    }
    
    // ... delete logic
}
```

---

## Technology Stack

### Backend Services
```yaml
Language: Rust 1.75+
  - Type safety
  - Memory safety
  - Zero-cost abstractions
  - Fearless concurrency

Web Framework: Axum 0.7
  - Async/await native
  - Tower middleware
  - Type-safe extractors
  - WebSocket support

gRPC: Tonic 0.12
  - Async gRPC
  - Protocol Buffers
  - HTTP/2
  - Streaming support

Database: PostgreSQL 16
  - ACID compliance
  - JSON support
  - Full-text search
  - Triggers & functions

ORM: SQLx 0.8
  - Compile-time query checking
  - Async prepared statements
  - Type-safe SQL
  - Migration management

Authentication: jsonwebtoken 9.0
  - HS256/RS256 algorithms
  - Claims validation
  - Token expiry

Messaging (Future): NATS 2.10
  - JetStream for persistence
  - At-least-once delivery
  - Consumer groups
  - Request-reply

Caching (Future): Redis 7.2
  - In-memory key-value
  - Pub/sub
  - Streams
  - Cluster mode
```

### Observability
```yaml
Logging: tracing + tracing-subscriber
  - Structured logging
  - Span tracking
  - Log levels
  - JSON output

Metrics: Prometheus + Grafana
  - Request duration
  - Error rates
  - Throughput
  - Custom business metrics

Tracing (Future): Jaeger
  - Distributed tracing
  - Span visualization
  - Performance profiling

Health Checks: Custom endpoints
  - /health/live (liveness)
  - /health/ready (readiness)
  - Dependency checks
```

### Infrastructure
```yaml
Containerization: Docker 24+
  - Multi-stage builds
  - Layer caching
  - Security scanning

Orchestration (Future): Kubernetes
  - Pod autoscaling
  - Service discovery
  - Load balancing
  - Rolling updates

CI/CD: GitHub Actions
  - Automated testing
  - Docker builds
  - Deployment pipelines
```

---

## Deployment Architecture

### MVP Deployment (Docker Compose)
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    
  auth-service:
    build: ./services/auth-service
    ports:
      - "8081:8081"
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://...
      JWT_SECRET: ${JWT_SECRET}
    
  user-service:
    build: ./services/user_profile-service
    ports:
      - "8082:8082"
      - "50051:50051"  # gRPC
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://...
      GRPC_PORT: 50051
```

### Production Deployment (Future - Kubernetes)
```yaml
apiVersion: apps/auth
kind: Deployment
metadata:
  name: user_profile-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: user_profile-service
  template:
    metadata:
      labels:
        app: user_profile-service
    spec:
      containers:
      - name: user_profile-service
        image: discord-clone/user_profile-service:auth.0.0
        ports:
        - containerPort: 8082
        - containerPort: 50051
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8082
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8082
          initialDelaySeconds: 5
          periodSeconds: 10
```

---

## Scalability Strategy

### Horizontal Scaling
```
┌─────────────────────────────────────────┐
│         Load Balancer                   │
└────────┬───────────┬────────────────────┘
         │           │
    ┌────▼────┐ ┌───▼─────┐ ┌──────────┐
    │User Svc │ │User Svc │ │User Svc  │
    │Pod 1    │ │Pod 2    │ │Pod 3     │
    └────┬────┘ └───┬─────┘ └────┬─────┘
         │          │            │
         └──────────┼────────────┘
                    │
              ┌─────▼─────┐
              │PostgreSQL │
              │Read Replicas
              └───────────┘
```

**Stateless Services:**
- No session state in memory
- JWT for authentication
- Shared database/cache
- Any pod can handle any request

**Database Scaling:**
1. Read replicas for query distribution
2. Connection pooling (PgBouncer)
3. Sharding (future - by user_id hash)

---

### Vertical Scaling

**Resource Optimization:**
```rust
// Connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)      // Limit connections
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;

// Async task limits
let semaphore = Arc::new(Semaphore::new(100));  // Max concurrent tasks
```

**Performance Targets:**

| Metric | Target | Current |
|--------|--------|---------|
| p50 Response Time | <50ms | TBD |
| p95 Response Time | <100ms | TBD |
| p99 Response Time | <200ms | TBD |
| Throughput | 10k req/s per pod | TBD |
| Database Connections | <100 per pod | 20 |

---

## Monitoring & Observability

### Metrics Collection
```rust
use prometheus::{register_histogram, register_counter};

lazy_static! {
    static ref HTTP_DURATION: Histogram = register_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration"
    ).unwrap();
    
    static ref HTTP_REQUESTS: Counter = register_counter!(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();
}

async fn handle_request() {
    let timer = HTTP_DURATION.start_timer();
    HTTP_REQUESTS.inc();
    
    // ... handle request
    
    timer.observe_duration();
}
```

### Key Metrics
```yaml
Service Health:
  - http_requests_total{method, status, service}
  - http_request_duration_seconds{method, path, service}
  - grpc_requests_total{method, status, service}
  - grpc_request_duration_seconds{method, service}

Business Metrics:
  - user_registrations_total
  - friend_requests_sent_total
  - friend_requests_accepted_total
  - active_users_total

Infrastructure:
  - db_connections_active
  - db_query_duration_seconds
  - cache_hits_total
  - cache_misses_total
```

### Logging Standards
```rust
#[instrument(skip(self), fields(user_id = %user_id))]
async fn send_friend_request(
    &self,
    user_id: Uuid,
    target_username: &str,
) -> Result<FriendRequest> {
    info!("Sending friend request");
    
    // ... logic
    
    if error {
        warn!("Friend request failed: {}", error);
        return Err(error);
    }
    
    info!("Friend request sent successfully");
    Ok(request)
}
```

**Log Levels:**
- **ERROR**: Requires immediate action
- **WARN**: Potentially harmful situations
- **INFO**: Informational messages
- **DEBUG**: Detailed debugging information
- **TRACE**: Very detailed tracing

---

## Security Architecture

### Defense in Depth
```
┌──────────────────────────────────────────┐
│  Layer 1: Network (Firewall, DDoS)      │
├──────────────────────────────────────────┤
│  Layer 2: API Gateway (Rate limit, WAF) │
├──────────────────────────────────────────┤
│  Layer 3: Authentication (JWT)          │
├──────────────────────────────────────────┤
│  Layer 4: Authorization (RBAC)          │
├──────────────────────────────────────────┤
│  Layer 5: Data (Encryption, Validation) │
└──────────────────────────────────────────┘
```

### Security Measures

**Authentication:**
- Argon2 password hashing
- JWT with short expiry
- Refresh token rotation
- Rate limiting on auth endpoints

**Data Protection:**
- SQL injection prevention (parameterized queries)
- XSS prevention (sanitized input)
- CORS configuration
- HTTPS only (TLS 1.3)

**Secrets Management:**
- Environment variables
- Vault (production)
- No secrets in code/git

---

**End of Architecture Documentation**