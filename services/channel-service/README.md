# Channel Service

Server (Guild) and channel management service for Hermes.

## Responsibilities

- Server (Guild) creation and management
- Channel creation (text, voice, category)
- Role and permission management
- Server invites
- Member management
- Bans and moderation
- Audit logs

## API Endpoints

### Servers

#### List User Servers
```http
GET /servers
Authorization: Bearer <token>
```

#### Create Server
```http
POST /servers
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Server",
  "icon_url": "https://cdn.example.com/icon.jpg",
  "region": "us-east"
}
```

#### Get Server
```http
GET /servers/{server_id}
Authorization: Bearer <token>
```

#### Update Server
```http
PATCH /servers/{server_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Name",
  "description": "A cool server"
}
```

#### Delete Server
```http
DELETE /servers/{server_id}
Authorization: Bearer <token>
```

### Channels

#### Create Channel
```http
POST /servers/{server_id}/channels
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "general",
  "type": "text",
  "topic": "General discussion",
  "parent_id": "uuid"
}
```

#### Update Channel
```http
PATCH /channels/{channel_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "new-name",
  "topic": "Updated topic"
}
```

#### Delete Channel
```http
DELETE /channels/{channel_id}
Authorization: Bearer <token>
```

### Roles

#### Create Role
```http
POST /servers/{server_id}/roles
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Moderator",
  "color": "#FF5733",
  "permissions": 8192,
  "mentionable": true
}
```

#### Update Role
```http
PATCH /roles/{role_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Admin",
  "permissions": 8
}
```

### Members

#### Get Server Members
```http
GET /servers/{server_id}/members?limit=100
Authorization: Bearer <token>
```

#### Add Member Role
```http
PUT /members/{member_id}/roles/{role_id}
Authorization: Bearer <token>
```

#### Kick Member
```http
DELETE /servers/{server_id}/members/{user_id}
Authorization: Bearer <token>
```

#### Ban Member
```http
PUT /servers/{server_id}/bans/{user_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "reason": "Spam",
  "delete_message_days": 7
}
```

### Invites

#### Create Invite
```http
POST /channels/{channel_id}/invites
Authorization: Bearer <token>
Content-Type: application/json

{
  "max_age": 86400,
  "max_uses": 10
}
```

#### Get Invite
```http
GET /invites/{invite_code}
```

#### Use Invite
```http
POST /invites/{invite_code}
Authorization: Bearer <token>
```

## Permission System

Permissions are stored as bitflags:

```rust
CREATE_INSTANT_INVITE = 1 << 0  // 1
KICK_MEMBERS = 1 << 1           // 2
BAN_MEMBERS = 1 << 2            // 4
ADMINISTRATOR = 1 << 3          // 8
MANAGE_CHANNELS = 1 << 4        // 16
MANAGE_SERVER = 1 << 5          // 32
SEND_MESSAGES = 1 << 11         // 2048
MANAGE_MESSAGES = 1 << 13       // 8192
CONNECT = 1 << 20               // 1048576
SPEAK = 1 << 21                 // 2097152
MUTE_MEMBERS = 1 << 22          // 4194304
```

## NATS Events

### Published Events
- `server.created` - New server created
- `server.updated` - Server settings changed
- `server.deleted` - Server deleted
- `channel.created` - New channel created
- `channel.updated` - Channel settings changed
- `channel.deleted` - Channel deleted
- `member.joined` - New member joined
- `member.left` - Member left/kicked
- `member.banned` - Member banned
- `role.created` - New role created
- `role.updated` - Role changed

### Subscribed Events
- `user.deleted` - Remove user from all servers

## Environment Variables

```bash
DATABASE_URL=postgres://hermes:password@localhost:5432/hermes
REDIS_URL=redis://:password@localhost:6379
NATS_URL=nats://localhost:4222
PORT=8083
```

## Running

```bash
cargo run --bin channel-service
```

Server starts on: http://localhost:8083
