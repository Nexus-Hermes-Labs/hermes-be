# Hermes - Development Roadmap

**Timeline:** 12 weeks (3 months)  
**Effort:** 6 hours per week  
**Total:** 72 hours

---

## 📊 Overview

| Phase       | Weeks | Focus                          | Status |
| ----------- | ----- | ------------------------------ | ------ |
| **Phase 1** | 1-4   | Infrastructure & Core Services | ☐      |
| **Phase 2** | 5-8   | Messaging & Real-time          | ☐      |
| **Phase 3** | 9-12  | Voice, Presence & Polish       | ☐      |

---

## Phase 1: Infrastructure & Core Services

### Week 1: Environment Setup

**Goal:** Get development environment ready

**Tasks:**

- [ ] Install Docker & Docker Compose
- [ ] Start PostgreSQL, Redis, NATS containers
- [ ] Verify container health
- [ ] Install Rust 1.75+ and cargo
- [ ] Setup IDE (VS Code + rust-analyzer)
- [ ] Build project: `cargo build --workspace`
- [ ] Create migrations folder structure
- [ ] Run first migration (users table)
- [ ] Load seed data (test users)
- [ ] Read ARCHITECTURE.md thoroughly

**Deliverables:**

- ✅ All infrastructure running
- ✅ Project builds successfully
- ✅ Database schema created
- ✅ Test data loaded

**Check:**

```bash
docker ps  # All containers healthy
cargo build --workspace  # Success
psql $DATABASE_URL -c "SELECT count(*) FROM users"  # Returns test users
```

---

### Week 2: Auth Service

**Goal:** Complete authentication system

**Tasks:**

- [ ] Implement `POST /register` endpoint
  - Input validation (email, password strength)
  - Argon2id password hashing
  - Insert user to PostgreSQL
  - Return JWT tokens
- [ ] Implement `POST /login` endpoint
  - Query user by email
  - Verify password with Argon2
  - Generate access token (1h) and refresh token (7d)
  - Store refresh token in Redis
- [ ] Implement `POST /refresh` endpoint
  - Validate refresh token
  - Generate new access token
- [ ] Implement `POST /logout` endpoint
  - Invalidate refresh token in Redis
- [ ] Add JWT middleware for token validation
- [ ] Write tests for all endpoints

**Deliverables:**

- ✅ User registration works
- ✅ Login returns valid JWT
- ✅ Token refresh works
- ✅ Protected endpoints validate JWT

**Check:**

```bash
curl -X POST localhost:8081/register -d '{"username":"test","email":"test@test.com","password":"Test123!"}'
curl -X POST localhost:8081/login -d '{"email":"test@test.com","password":"Test123!"}'
```

---

### Week 3: User Service

**Goal:** User profiles and friend system

**Tasks:**

- [ ] Implement `GET /users/@me` - Current user profile
- [ ] Implement `PATCH /users/@me` - Update profile
  - display_name, avatar_url, bio
- [ ] Implement `GET /users/:id` - Get user by ID
- [ ] Implement `GET /users/search?q=username` - Search users
- [ ] Implement friend system:
  - `POST /users/@me/friends` - Send friend request
  - `GET /users/@me/friends` - List friends
  - `DELETE /users/@me/friends/:id` - Remove friend
  - `POST /users/@me/friends/:id/accept` - Accept request
- [ ] Add Redis caching for user profiles
- [ ] Publish NATS events (user.profile.updated, user.friend.added)

**Deliverables:**

- ✅ Profile CRUD works
- ✅ Friend system functional
- ✅ User search works
- ✅ Redis caching active

**Check:**

```bash
curl -H "Authorization: Bearer $TOKEN" localhost:8082/users/@me
curl -H "Authorization: Bearer $TOKEN" localhost:8082/users/search?q=alice
```

---

### Week 4: Channel Service Part 1

**Goal:** Server and channel management

**Tasks:**

- [ ] Create migrations for servers, channels, roles, members
- [ ] Implement server endpoints:
  - `POST /servers` - Create server
  - `GET /servers` - List user's servers
  - `GET /servers/:id` - Server details
  - `PATCH /servers/:id` - Update server
  - `DELETE /servers/:id` - Delete server
- [ ] Implement channel endpoints:
  - `POST /servers/:id/channels` - Create channel
  - `GET /channels/:id` - Channel details
  - `PATCH /channels/:id` - Update channel
  - `DELETE /channels/:id` - Delete channel
- [ ] Implement member management:
  - `POST /servers/:id/join` - Join server
  - `GET /servers/:id/members` - List members
  - `DELETE /servers/:id/members/:id` - Kick member
- [ ] Publish NATS events (server.created, channel.created)

**Deliverables:**

- ✅ Server CRUD works
- ✅ Channel CRUD works
- ✅ Member management works
- ✅ NATS events published

**Check:**

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" localhost:8083/servers \
  -d '{"name":"Test Server"}'
```

---

## Phase 2: Messaging & Real-time

### Week 5: Chat Service Part 1

**Goal:** Basic messaging

**Tasks:**

- [ ] Create migrations for messages, reactions, attachments
- [ ] Implement `POST /channels/:id/messages` - Send message
  - Validate content (not empty, max 2000 chars)
  - Insert to PostgreSQL
  - Cache in Redis (last 50 messages)
  - Publish NATS event (message.created)
- [ ] Implement `GET /channels/:id/messages` - Message history
  - Pagination with before/after message_id
  - Limit parameter (default 50, max 100)
  - Order by created_at DESC
- [ ] Implement `PATCH /messages/:id` - Edit message
  - Permission check (author or admin)
  - Update edited_at timestamp
- [ ] Implement `DELETE /messages/:id` - Delete message
  - Permission check
  - Soft delete or hard delete
- [ ] Add message validation and sanitization

**Deliverables:**

- ✅ Send message works
- ✅ Message history with pagination
- ✅ Edit/delete messages
- ✅ NATS events for real-time

**Check:**

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  localhost:8084/channels/$CHANNEL_ID/messages \
  -d '{"content":"Hello, world!"}'
```

---

### Week 6: Chat Service Part 2

**Goal:** Reactions and mentions

**Tasks:**

- [ ] Implement reactions:
  - `POST /messages/:id/reactions/:emoji` - Add reaction
  - `DELETE /messages/:id/reactions/:emoji` - Remove reaction
  - Get messages with reaction counts
- [ ] Implement mentions:
  - Parse @username in message content
  - Store mentioned user IDs
  - Query users who were mentioned
- [ ] Implement direct messages:
  - Create DM channel between two users
  - Send DM endpoint
  - List DM conversations
- [ ] Add message search (basic)
- [ ] Optimize queries with indexes

**Deliverables:**

- ✅ Reactions work
- ✅ Mentions parsed and stored
- ✅ DMs functional
- ✅ Message search works

**Check:**

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  localhost:8084/messages/$MSG_ID/reactions/👍
```

---

### Week 7: Gateway Service Part 1

**Goal:** WebSocket connection and event handling

**Tasks:**

- [ ] Implement WebSocket upgrade handler
- [ ] Add JWT authentication for WebSocket
  - Validate token from query param or header
  - Reject invalid connections
- [ ] Implement heartbeat system
  - Client sends heartbeat every 30s
  - Server responds with ACK
  - Disconnect if no heartbeat for 60s
- [ ] Implement event handling:
  - Parse client op codes
  - IDENTIFY event (connection start)
  - READY event (send user info)
  - MESSAGE_CREATE (send message)
- [ ] Subscribe to NATS events:
  - message.created, message.updated, message.deleted
  - Forward to connected clients
- [ ] Implement connection state management
  - Track active connections
  - User → Connection mapping

**Deliverables:**

- ✅ WebSocket server running
- ✅ Authentication works
- ✅ Heartbeat system active
- ✅ Events forwarded from NATS

**Check:**

```javascript
// Browser console
const ws = new WebSocket("ws://localhost:8080/ws?token=JWT_TOKEN");
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

---

### Week 8: Gateway Service Part 2

**Goal:** API routing and rate limiting

**Tasks:**

- [ ] Implement API proxy routing:
  - `/api/auth/*` → Auth Service (8081)
  - `/api/users/*` → User Service (8082)
  - `/api/servers/*` → Channel Service (8083)
  - `/api/channels/*` → Chat Service (8084)
  - `/api/messages/*` → Chat Service (8084)
- [ ] Add HTTP client (reqwest) for service calls
- [ ] Forward auth headers to services
- [ ] Implement rate limiting:
  - Message rate: 10 per 10 seconds per user
  - API rate: 60 per minute per user
  - WebSocket frames: 120 per minute
  - Use Redis for rate limit counters
- [ ] Add connection manager:
  - Track all active connections
  - Broadcast events to specific users
  - Handle disconnections gracefully
- [ ] Error handling and logging

**Deliverables:**

- ✅ API proxy works
- ✅ Rate limiting active
- ✅ Connection management complete
- ✅ Error handling robust

**Check:**

```bash
# All API calls work through gateway
curl localhost:8080/api/users/@me -H "Authorization: Bearer $TOKEN"
curl localhost:8080/api/servers -H "Authorization: Bearer $TOKEN"
```

---

## Phase 3: Voice, Presence & Polish

### Week 9: Presence Service

**Goal:** Online status and typing indicators

**Tasks:**

- [ ] Implement status management:
  - `POST /presence/status` - Update status
  - online, idle, dnd, offline
  - Custom status messages
  - Store in Redis with 5min TTL
- [ ] Implement heartbeat system:
  - Gateway → Presence heartbeat every 30s
  - Update presence TTL
  - Auto-offline if heartbeat stops
- [ ] Implement typing indicators:
  - `POST /presence/typing` - Start typing
  - Store in Redis sorted set (10s TTL)
  - Broadcast typing event via NATS
  - Auto-cleanup expired indicators
- [ ] Implement bulk presence queries:
  - `POST /presence/bulk` - Get multiple presences
  - Use Redis pipeline for efficiency
- [ ] Publish NATS events (presence.status.changed)

**Deliverables:**

- ✅ Presence tracking works
- ✅ Heartbeat system active
- ✅ Typing indicators functional
- ✅ Bulk queries optimized

**Check:**

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  localhost:8087/presence/status \
  -d '{"status":"online","custom_status":"Coding 🦀"}'
```

---

### Week 10: Voice Service Part 1

**Goal:** WebRTC signaling foundation

**Tasks:**

- [ ] Create voice_sessions migration
- [ ] Implement `POST /voice/join` endpoint:
  - Create voice session in PostgreSQL
  - Return ICE servers (public STUN)
  - Check 2-user limit per channel
  - Return peer_id if another user in channel
- [ ] Implement `POST /voice/signal` endpoint:
  - Accept SDP offer/answer
  - Accept ICE candidates
  - Forward to other peer in channel
  - Store signaling state in Redis
- [ ] Implement `POST /voice/leave` endpoint:
  - Update session with left_at
  - Notify other user
  - Cleanup Redis state
- [ ] Implement voice state updates:
  - `PATCH /voice/state` - Mute/unmute
  - Update session in database
  - Broadcast state change via NATS

**Deliverables:**

- ✅ Voice session management
- ✅ WebRTC signaling works
- ✅ Join/leave functional
- ✅ State updates work

**Check:**

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  localhost:8085/voice/join \
  -d '{"channel_id":"$CHANNEL_ID"}'
```

---

### Week 11: Voice Service Part 2 + Client

**Goal:** Complete P2P voice calling

**Tasks:**

- [ ] Implement session management:
  - List active sessions in channel
  - Get session by ID
  - Session timeout handling (if user disconnects without leaving)
  - Cleanup stale sessions
- [ ] Create simple HTML/JS voice client:
  - WebSocket connection to gateway
  - Join voice channel
  - Create WebRTC PeerConnection
  - getUserMedia() for audio
  - Handle ICE candidates
  - Send/receive SDP via signaling
  - Display connection status
  - Mute/unmute button
- [ ] Test voice calling:
  - Two browser tabs
  - Join same voice channel
  - Establish P2P connection
  - Verify audio works
- [ ] Handle edge cases:
  - Network interruptions
  - ICE failure fallback
  - Connection timeout

**Deliverables:**

- ✅ Session management complete
- ✅ Simple voice client works
- ✅ P2P audio connection established
- ✅ 2-user voice call functional

**Check:** Open two browser tabs, join same voice channel, hear each other!

---

### Week 12: Testing & Polish

**Goal:** Production-ready MVP

**Tasks:**

- [ ] Write integration tests:
  - Auth flow (register → login → refresh)
  - Message flow (send → receive → edit → delete)
  - Friend flow (request → accept → list)
  - Voice flow (join → signal → leave)
- [ ] Write unit tests for core logic:
  - Password hashing
  - JWT generation/validation
  - Permission calculations
  - Rate limiting
- [ ] Bug fixes:
  - Fix any remaining bugs
  - Handle edge cases
  - Improve error messages
- [ ] Performance optimization:
  - Add missing indexes
  - Optimize slow queries
  - Reduce Redis cache misses
  - Profile and fix bottlenecks
- [ ] Documentation:
  - Update API documentation
  - Write deployment guide
  - Create troubleshooting guide
  - Record demo video
- [ ] Demo preparation:
  - Test full user flow
  - Prepare screenshots
  - Create portfolio write-up

**Deliverables:**

- ✅ Tests passing
- ✅ Known bugs fixed
- ✅ Performance acceptable
- ✅ Documentation complete
- ✅ Demo ready

**Check:**

```bash
cargo test --workspace  # All tests pass
cargo clippy  # No warnings
make test-flow  # End-to-end tests pass
```

---

## 🎯 Milestones

### Milestone 1: Infrastructure Ready (Week 1)

- [ ] Docker containers running
- [ ] Database migrated
- [ ] Project builds
- [ ] Dev environment complete

### Milestone 2: Core Services (Week 4)

- [ ] Auth works
- [ ] Users and friends
- [ ] Servers and channels
- [ ] Basic functionality

### Milestone 3: Real-time Messaging (Week 8)

- [ ] Chat functional
- [ ] WebSocket gateway
- [ ] Real-time updates
- [ ] Rate limiting

### Milestone 4: MVP Complete (Week 12)

- [ ] Presence tracking
- [ ] P2P voice calls
- [ ] Tests passing
- [ ] Production-ready

---

## 📊 Weekly Effort Breakdown

| Activity      | Hours/Week |
| ------------- | ---------- |
| Coding        | 4-5h       |
| Testing       | 0.5-1h     |
| Documentation | 0.5h       |
| Learning      | Ongoing    |

**Total:** 6 hours/week

---

## 🚀 Success Criteria

By week 12, you should have:

✅ **7 microservices** running  
✅ **PostgreSQL** with complete schema  
✅ **Redis** caching and pub/sub  
✅ **NATS** event streaming  
✅ **WebSocket** real-time messaging  
✅ **P2P Voice** (2-user calls)  
✅ **Tests** for core functionality  
✅ **Documentation** complete  
✅ **Deployable** to production

---

## 🎓 Learning Outcomes

You will have learned:

- Microservice architecture design
- Event-driven systems with NATS
- Real-time WebSocket communication
- PostgreSQL schema design
- Redis caching strategies
- JWT authentication
- WebRTC P2P basics
- Rust web development with Axum
- SQLx compile-time queries
- Docker deployment
- API design and documentation

---

## 💡 Tips for Success

### Stay on Track

- Set aside dedicated time each week
- Use a timer (Pomodoro technique)
- Complete week's tasks before moving on
- Don't skip testing

### When Stuck

- Read documentation (Axum, SQLx, WebRTC)
- Check examples in repos
- Ask for help
- Take a break and come back fresh

### Best Practices

- Commit code regularly (daily)
- Write tests as you go
- Keep notes of decisions
- Document tricky parts

### Avoid

- Skipping weeks (momentum is important)
- Overengineering (stick to MVP)
- Ignoring errors (fix as you go)
- Working without tests

---

## 🔄 Flexibility

This roadmap is a guide, not a strict rule. Feel free to:

- Adjust pace based on your speed
- Swap week order if needed
- Spend extra time on difficult parts
- Skip features that aren't critical
- Add features that excite you

**Remember:** Done is better than perfect!

---

## 📈 Progress Tracking

Mark your progress:

**Week 1:** ☐  
**Week 2:** ☐  
**Week 3:** ☐  
**Week 4:** ☐  
**Week 5:** ☐  
**Week 6:** ☐  
**Week 7:** ☐  
**Week 8:** ☐  
**Week 9:** ☐  
**Week 10:** ☐  
**Week 11:** ☐  
**Week 12:** ☐

**Started:** ****\_\_\_****  
**Completed:** ****\_\_\_****

---

## 🎉 Celebration

When you complete week 12:

1. Deploy your app!
2. Share on Twitter/LinkedIn
3. Add to portfolio
4. Update resume
5. Start job applications
6. Plan next project

**You built a Discord clone in Rust! 🦀🚀**

---

_Ready to start? Go to Week 1 and let's build! 💪_
