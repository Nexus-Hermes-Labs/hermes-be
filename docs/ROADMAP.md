# Discord Clone - Development Roadmap

**Project Start:** Week 1  
**Current Week:** Week 3, Day 2  
**MVP Target:** Week 12  
**Last Updated:** February 7, 2026

## Table of Contents

- [Project Overview](#project-overview)
- [Current Status](#current-status)
- [Week-by-Week Plan](#week-by-week-plan)
- [Implementation Status](#implementation-status)
- [Technical Debt](#technical-debt)
- [Risk Register](#risk-register)

---

## Project Overview

### MVP Scope (12 Weeks)

Building core Discord functionality:
- ✅ User authentication & authorization
- ✅ User profiles with discriminators
- ✅ Friend system (requests, accept/decline)
- ✅ Block system
- ⏳ Direct messaging
- ⏳ Servers & channels
- ⏳ Real-time messaging
- ⏳ Voice channels (stretch)

### Architecture Decision

**Communication Strategy:**
- **gRPC**: Service-to-service sync operations (MVP)
- **HTTP REST**: Client-to-service APIs
- **NATS**: Async events (Post-MVP)

---

## Current Status

### 🎯 Week 3, Day 2 - User Service Development

**Current Sprint:**
- ✅ Domain layer complete (User, UserRelationship)
- ✅ Repository traits defined
- ✅ Application services complete
- ⏳ gRPC implementation (IN PROGRESS)
- ⏳ Infrastructure repositories
- ⏳ HTTP endpoints

**Current Focus:**
1. Complete gRPC service definitions
2. Implement User Service gRPC server
3. Implement Auth Service gRPC client
4. Infrastructure repositories

---

## Week-by-Week Plan

### ✅ Week 1: Project Setup & Auth Service (COMPLETE)

**Goals:**
- [x] Project structure
- [x] Database setup (PostgreSQL)
- [x] Common library (errors, pagination, repository trait)
- [x] Auth Service domain layer
- [x] JWT authentication
- [x] Password hashing (Argon2)

**Deliverables:**
- [x] `POST /v1/auth/register`
- [x] `POST /v1/auth/login`
- [x] `POST /v1/auth/refresh`
- [x] `POST /v1/auth/logout`
- [x] JWT middleware

**Time Spent:** 5 days  
**Status:** ✅ Complete

---

### ✅ Week 2: User Service Foundation (COMPLETE)

**Goals:**
- [x] Users table migration
- [x] User domain entity
- [x] Privacy settings (DM, friend requests, online status)
- [x] Custom status
- [x] User repository trait
- [x] Basic user endpoints

**Deliverables:**
- [x] Database schema for users
- [x] User domain model
- [x] Privacy enums (DmPrivacy, FriendRequestPrivacy)
- [x] Custom status value object

**Time Spent:** 5 days  
**Status:** ✅ Complete

---

### ⏳ Week 3: User Relationships & gRPC (IN PROGRESS)

#### Days 1-2: Domain Layer ✅ COMPLETE

**Completed:**
- [x] user_relationships table migration
- [x] Bidirectional sync trigger
- [x] Relationship type enum (Friend, Blocked, Pending)
- [x] UserRelationship entity
- [x] UserRelationshipDomainError (comprehensive)
- [x] RelationshipType value object
- [x] UserRelationshipDomainService
- [x] UserRelationshipApplicationService
- [x] UserRelationshipApplicationError
- [x] Repository traits (UserRepository, UserRelationshipRepository)

**Artifacts:**
- Domain entities with full business logic
- Factory methods (create_friend_request, create_block)
- State transitions (accept, decline, cancel)
- Privacy validation
- Friends-of-friends checking

---

#### Days 3-4: gRPC Implementation ⏳ IN PROGRESS

**Tasks:**

**Day 3 - Morning (2 hours):**
- [ ] Proto definitions (`proto/user_service.proto`)
  - [ ] UserService service definition
  - [ ] GenerateDiscriminator RPC
  - [ ] CheckUsernameAvailability RPC
  - [ ] GetUserById RPC
  - [ ] GetUserByEmail RPC
  - [ ] UpdateUsername RPC
  - [ ] AreFriends RPC
  - [ ] IsBlocked RPC
  - [ ] Message definitions (Request/Response)
  - [ ] User message (common type)

**Day 3 - Afternoon (3 hours):**
- [ ] User Service gRPC server
  - [ ] `src/presentation/grpc/user_service.rs`
  - [ ] Implement UserService trait
  - [ ] Wire up discriminator service
  - [ ] Wire up user application service
  - [ ] Error mapping (domain → gRPC status)
  - [ ] Start gRPC server in main.rs (port 50051)

**Day 4 - Morning (2 hours):**
- [ ] Auth Service gRPC client
  - [ ] `src/infrastructure/grpc/user_client.rs`
  - [ ] UserServiceGrpcClient struct
  - [ ] Connection management
  - [ ] Retry logic
  - [ ] Error handling
  
**Day 4 - Afternoon (2 hours):**
- [ ] Update registration flow
  - [ ] Use gRPC client in RegistrationService
  - [ ] Generate discriminator via gRPC
  - [ ] Check availability via gRPC
  - [ ] Error handling
  - [ ] Integration testing

**Deliverables:**
- [ ] Working gRPC communication
- [ ] Auth Service can call User Service
- [ ] Discriminator generation works end-to-end

---

#### Day 5: Infrastructure Repositories (6 hours)

**Tasks:**

**PostgresUserRepository (3 hours):**
- [ ] `src/infrastructure/persistence/user/repository.rs`
- [ ] Implement base Repository trait
  - [ ] find_by_id
  - [ ] find_all (admin only - future)
  - [ ] save
  - [ ] update
  - [ ] delete
  - [ ] exists
  - [ ] count
- [ ] Implement UserRepository trait
  - [ ] find_by_username
  - [ ] find_by_email
  - [ ] find_by_ids (bulk fetch)
  - [ ] search (username search)
- [ ] Error handling (SQLx → RepositoryError)
- [ ] Row mapping (PostgreSQL → Domain entity)

**PostgresDiscriminatorRepository (1 hour):**
- [ ] `src/infrastructure/persistence/discriminator/repository.rs`
- [ ] find_max_discriminator
- [ ] exists (username + discriminator)
- [ ] count_by_username

**PostgresUserRelationshipRepository (2 hours):**
- [ ] `src/infrastructure/persistence/user_relationship/repository.rs`
- [ ] Implement base Repository trait (7 methods)
- [ ] Implement UserRelationshipRepository trait
  - [ ] find_relationship
  - [ ] find_friends (paginated)
  - [ ] find_pending_incoming (paginated)
  - [ ] find_pending_outgoing (paginated)
  - [ ] find_blocked (paginated)
  - [ ] count_friends
  - [ ] count_pending_incoming
  - [ ] count_pending_outgoing
  - [ ] count_blocked
  - [ ] are_friends (boolean)
  - [ ] is_blocked (boolean)
  - [ ] relationship_exists (boolean)
  - [ ] delete_relationship
- [ ] Test bidirectional triggers

**Testing:**
- [ ] Integration tests with test database
- [ ] Verify trigger behavior
- [ ] Test all repository methods

**Deliverables:**
- [ ] Complete repository implementations
- [ ] All 20+ repository methods working
- [ ] Trigger validation

---

### 📅 Week 4: HTTP Endpoints & Integration

#### Days 1-2: User Profile Endpoints

**User Endpoints (Day 1 - 4 hours):**
- [ ] GET /v1/users/me
- [ ] PATCH /v1/users/me
- [ ] PUT /v1/users/me/status
- [ ] DELETE /v1/users/me/status
- [ ] PATCH /v1/users/me/privacy
- [ ] GET /v1/users/search

**Profile DTOs:**
- [ ] UserProfileResponse
- [ ] UpdateProfileRequest
- [ ] SetCustomStatusRequest
- [ ] UpdatePrivacyRequest
- [ ] UserSearchResponse

**Testing (Day 2 - 2 hours):**
- [ ] Unit tests for handlers
- [ ] Integration tests
- [ ] OpenAPI documentation

---

#### Days 3-4: Friend Relationship Endpoints

**Friend Request Endpoints (Day 3 - 4 hours):**
- [ ] POST /v1/friends/requests
- [ ] GET /v1/friends/requests/incoming
- [ ] GET /v1/friends/requests/outgoing
- [ ] POST /v1/friends/requests/{user_id}/accept
- [ ] POST /v1/friends/requests/{user_id}/decline
- [ ] DELETE /v1/friends/requests/{user_id} (cancel)

**Friend Management (Day 4 - 2 hours):**
- [ ] GET /v1/friends
- [ ] DELETE /v1/friends/{user_id} (unfriend)
- [ ] GET /v1/friends/count

**DTOs:**
- [ ] SendFriendRequestRequest
- [ ] FriendRequestResponse
- [ ] FriendListResponse
- [ ] Pagination support

**Testing (Day 4 - 2 hours):**
- [ ] E2E friend request flow
- [ ] Privacy validation tests
- [ ] Error case testing

---

#### Day 5: Block System Endpoints

**Block Endpoints (3 hours):**
- [ ] POST /v1/blocked
- [ ] GET /v1/blocked
- [ ] DELETE /v1/blocked/{user_id}

**DTOs:**
- [ ] BlockUserRequest
- [ ] BlockedUserResponse

**Testing (2 hours):**
- [ ] Block flow tests
- [ ] Side effect validation (unfriend on block)

**Documentation (1 hour):**
- [ ] Complete OpenAPI spec
- [ ] Update API_REFERENCE.md
- [ ] Postman collection

---

### 📅 Week 5: Testing, Observability & Polish

#### Days 1-2: Comprehensive Testing

**Unit Tests:**
- [ ] Domain layer coverage (>90%)
- [ ] Application service coverage (>80%)
- [ ] Repository tests

**Integration Tests:**
- [ ] Database integration
- [ ] gRPC integration
- [ ] HTTP endpoint tests

**E2E Tests:**
- [ ] Complete user flows
  - [ ] Registration → Login
  - [ ] Friend request → Accept → Unfriend
  - [ ] Block → Unblock
  - [ ] Privacy settings enforcement

---

#### Days 3-4: Observability

**Logging:**
- [ ] Structured logging with tracing
- [ ] Request ID tracking
- [ ] Log levels properly set
- [ ] JSON output for production

**Metrics:**
- [ ] Prometheus metrics
  - [ ] HTTP request duration
  - [ ] gRPC request duration
  - [ ] Database query duration
  - [ ] Error rates
  - [ ] Business metrics (registrations, friend requests)
- [ ] Grafana dashboards

**Health Checks:**
- [ ] /health/live endpoint
- [ ] /health/ready endpoint
- [ ] Database connectivity check
- [ ] gRPC service check

---

#### Day 5: Production Readiness

**Performance:**
- [ ] Query optimization
- [ ] Index validation
- [ ] Connection pool tuning
- [ ] Load testing (k6)

**Security:**
- [ ] Security audit
- [ ] Rate limiting implementation
- [ ] Input validation review
- [ ] Secrets management

**Documentation:**
- [ ] API documentation complete
- [ ] Architecture documentation
- [ ] Deployment guide
- [ ] Troubleshooting guide

---

### 📅 Weeks 6-8: Server Service (TBD)

**Scope:**
- Server creation & management
- Channels (text, voice)
- Roles & permissions
- Member management

**APIs:**
- Server CRUD
- Channel CRUD
- Member management
- Role management

---

### 📅 Weeks 9-10: Messaging Service (TBD)

**Scope:**
- Direct messages (1-to-1)
- Group DMs
- Channel messages
- Message history
- Attachments

**Real-time:**
- WebSocket connections
- Message delivery
- Typing indicators
- Read receipts

---

### 📅 Weeks 11-12: Integration & Polish (TBD)

**Scope:**
- End-to-end testing
- Performance optimization
- Bug fixes
- Documentation
- Demo preparation

---

## Implementation Status

### Service Completion Matrix

| Service | Domain | Application | Infrastructure | Presentation | Tests | Status |
|---------|--------|-------------|----------------|--------------|-------|--------|
| **Auth Service** | ✅ | ✅ | ✅ | ✅ | ✅ | **Complete** |
| **User Service** | ✅ | ✅ | ⏳ | ⏳ | ❌ | **60%** |
| **Server Service** | ❌ | ❌ | ❌ | ❌ | ❌ | **0%** |
| **Message Service** | ❌ | ❌ | ❌ | ❌ | ❌ | **0%** |

---

### Endpoint Implementation Status

#### Auth Service ✅ COMPLETE

| Method | Endpoint | Status | Tests |
|--------|----------|--------|-------|
| POST | /v1/auth/register | ✅ | ✅ |
| POST | /v1/auth/login | ✅ | ✅ |
| POST | /v1/auth/refresh | ✅ | ✅ |
| POST | /v1/auth/logout | ✅ | ✅ |

---

#### User Service ⏳ IN PROGRESS

**Profile Endpoints:**

| Method | Endpoint | Domain | Application | Handler | Tests | Status |
|--------|----------|--------|-------------|---------|-------|--------|
| GET | /v1/users/me | ✅ | ✅ | ❌ | ❌ | 50% |
| PATCH | /v1/users/me | ✅ | ✅ | ❌ | ❌ | 50% |
| PUT | /v1/users/me/status | ✅ | ✅ | ❌ | ❌ | 50% |
| DELETE | /v1/users/me/status | ✅ | ✅ | ❌ | ❌ | 50% |
| PATCH | /v1/users/me/privacy | ✅ | ✅ | ❌ | ❌ | 50% |
| GET | /v1/users/search | ✅ | ✅ | ❌ | ❌ | 50% |

**Friend Request Endpoints:**

| Method | Endpoint | Domain | Application | Handler | Tests | Status |
|--------|----------|--------|-------------|---------|-------|--------|
| POST | /v1/friends/requests | ✅ | ✅ | ❌ | ❌ | 50% |
| GET | /v1/friends/requests/incoming | ✅ | ✅ | ❌ | ❌ | 50% |
| GET | /v1/friends/requests/outgoing | ✅ | ✅ | ❌ | ❌ | 50% |
| POST | /v1/friends/requests/:id/accept | ✅ | ✅ | ❌ | ❌ | 50% |
| POST | /v1/friends/requests/:id/decline | ✅ | ✅ | ❌ | ❌ | 50% |
| DELETE | /v1/friends/requests/:id | ✅ | ✅ | ❌ | ❌ | 50% |

**Friend Management:**

| Method | Endpoint | Domain | Application | Handler | Tests | Status |
|--------|----------|--------|-------------|---------|-------|--------|
| GET | /v1/friends | ✅ | ✅ | ❌ | ❌ | 50% |
| DELETE | /v1/friends/:id | ✅ | ✅ | ❌ | ❌ | 50% |
| GET | /v1/friends/count | ✅ | ✅ | ❌ | ❌ | 50% |

**Block System:**

| Method | Endpoint | Domain | Application | Handler | Tests | Status |
|--------|----------|--------|-------------|---------|-------|--------|
| POST | /v1/blocked | ✅ | ✅ | ❌ | ❌ | 50% |
| GET | /v1/blocked | ✅ | ✅ | ❌ | ❌ | 50% |
| DELETE | /v1/blocked/:id | ✅ | ✅ | ❌ | ❌ | 50% |

**Total:** 18 endpoints - 0 complete, 18 in progress

---

### gRPC Service Status

**User Service gRPC:**

| RPC | Request | Response | Server | Client | Tests | Status |
|-----|---------|----------|--------|--------|-------|--------|
| GenerateDiscriminator | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| CheckUsernameAvailability | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| GetUserById | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| GetUserByEmail | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| UpdateUsername | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| AreFriends | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |
| IsBlocked | ⏳ | ⏳ | ⏳ | ⏳ | ❌ | 0% |

**Priority:** HIGH - Blocking Auth Service integration

---

## Technical Debt

### Current Debt Items

| Item | Severity | Impact | Effort | Target Week |
|------|----------|--------|--------|-------------|
| Missing HTTP endpoints | High | Blocks MVP | 1 week | Week 4 |
| No integration tests | Medium | Quality risk | 2 days | Week 5 |
| No observability | Medium | Ops risk | 2 days | Week 5 |
| Shared database | Low | Scale risk | 2 weeks | Post-MVP |
| No caching | Low | Perf risk | 1 week | Post-MVP |

### Planned Improvements

**Week 5:**
- Implement comprehensive testing
- Add observability (metrics, tracing)
- Performance benchmarking

**Post-MVP:**
- Split databases (service per database)
- Add Redis caching
- Add NATS for events
- Implement rate limiting
- Add API Gateway

---

## Risk Register

### Active Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| gRPC integration complexity | Medium | High | Start early, comprehensive testing |
| Database trigger bugs | Medium | High | Extensive testing, manual verification |
| Performance issues at scale | Low | High | Load testing in Week 5 |
| Missing features for MVP | Low | Medium | Strict scope control |

### Resolved Risks

| Risk | Resolution | Date |
|------|-----------|------|
| Auth complexity | JWT + Argon2 working well | Week 1 |
| Domain modeling complexity | DDD approach successful | Week 2 |
| Bidirectional relationship sync | Database triggers solved it | Week 3 |

---

## Success Metrics

### Week 3 Goals

**Must Have:**
- [x] Domain layer complete
- [x] Application services complete
- [ ] gRPC working end-to-end
- [ ] Repositories implemented

**Nice to Have:**
- [ ] Some HTTP endpoints
- [ ] Basic tests

### Week 4 Goals

**Must Have:**
- [ ] All HTTP endpoints implemented
- [ ] End-to-end friend request flow
- [ ] Basic testing

**Nice to Have:**
- [ ] Comprehensive tests
- [ ] Performance benchmarks

### Week 5 Goals

**Must Have:**
- [ ] >80% test coverage
- [ ] Observability implemented
- [ ] Production-ready

**Nice to Have:**
- [ ] Load testing complete
- [ ] Documentation polished

---

## Next Actions (This Week)

### Immediate (Today)

1. ✅ Create documentation (API Reference, Architecture, Roadmap)
2. ⏳ Define proto files (user_service.proto)
3. ⏳ Implement gRPC server (User Service)
4. ⏳ Implement gRPC client (Auth Service)

### Tomorrow

5. ⏳ Test gRPC integration
6. ⏳ Start repository implementations
7. ⏳ PostgresUserRepository

### This Week

8. ⏳ Complete all repositories
9. ⏳ Integration testing
10. ⏳ Document gRPC usage

---

**Last Updated:** February 7, 2026, Week 3 Day 2  
**Next Review:** February 14, 2026, Week 4 Day 2