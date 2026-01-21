# User Service

User profile and relationship management service for Hermes.

## Responsibilities

- User profile management
- Friend system (add, accept, block)
- User settings and preferences
- Avatar and banner management
- Bio and custom status
- User search

## API Endpoints

### Get Current User
```http
GET /users/@me
Authorization: Bearer <token>
```

### Update Profile
```http
PATCH /users/@me
Authorization: Bearer <token>
Content-Type: application/json

{
  "display_name": "New Name",
  "bio": "Hello, I'm using Hermes!",
  "avatar_url": "https://cdn.example.com/avatar.jpg"
}
```

### Get User by ID
```http
GET /users/{user_id}
Authorization: Bearer <token>
```

### Search Users
```http
GET /users/search?q=username&limit=20
Authorization: Bearer <token>
```

### Friends Management

#### Get Friends
```http
GET /users/@me/friends
Authorization: Bearer <token>
```

#### Send Friend Request
```http
POST /users/@me/friends
Authorization: Bearer <token>
Content-Type: application/json

{
  "user_id": "uuid"
}
```

#### Accept Friend Request
```http
PUT /users/@me/friends/{user_id}
Authorization: Bearer <token>
```

#### Remove Friend
```http
DELETE /users/@me/friends/{user_id}
Authorization: Bearer <token>
```

#### Block User
```http
POST /users/@me/blocked
Authorization: Bearer <token>
Content-Type: application/json

{
  "user_id": "uuid"
}
```

## NATS Events

### Published Events
- `user.profile.updated` - Profile changed
- `user.friend.added` - New friend
- `user.friend.removed` - Friend removed
- `user.blocked` - User blocked

### Subscribed Events
- `auth.user.created` - Initialize new user profile
- `presence.status.changed` - Update cached status

## Environment Variables

```bash
DATABASE_URL=postgres://hermes:password@localhost:5432/hermes
REDIS_URL=redis://:password@localhost:6379
NATS_URL=nats://localhost:4222
MINIO_ENDPOINT=localhost:9000
PORT=8082
```

## Running

```bash
cargo run --bin user-service
```

Server starts on: http://localhost:8082
