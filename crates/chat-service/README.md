# Chat Service

Text messaging service for Hermes.

## Responsibilities

- Send/receive text messages
- Message history and pagination
- Message editing and deletion
- Message reactions
- File attachments
- Message search
- Pinned messages
- Mentions and notifications

## Database Tables

- `messages` - All text messages
- `message_attachments` - File uploads
- `reactions` - Message reactions

## API Endpoints

### Get Messages
```http
GET /channels/{channel_id}/messages?limit=50&before={message_id}
Authorization: Bearer <token>
```

### Send Message
```http
POST /channels/{channel_id}/messages
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "Hello, world!",
  "attachments": ["url1", "url2"]
}
```

### Edit Message
```http
PATCH /messages/{message_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "Updated message"
}
```

### Delete Message
```http
DELETE /messages/{message_id}
Authorization: Bearer <token>
```

### Add Reaction
```http
PUT /messages/{message_id}/reactions/{emoji}
Authorization: Bearer <token>
```

### Remove Reaction
```http
DELETE /messages/{message_id}/reactions/{emoji}
Authorization: Bearer <token>
```

## NATS Events

### Published Events
- `message.created` - New message sent
- `message.updated` - Message edited
- `message.deleted` - Message deleted
- `reaction.added` - Reaction added
- `reaction.removed` - Reaction removed

### Subscribed Events
- `user.banned` - Remove messages from banned user
- `channel.deleted` - Clean up channel messages

## Environment Variables

```bash
DATABASE_URL=postgres://hermes:password@localhost:5432/hermes
REDIS_URL=redis://:password@localhost:6379
NATS_URL=nats://localhost:4222
MINIO_ENDPOINT=localhost:9000
PORT=8084
```

## Running

```bash
cargo run --bin chat-service
```

Server starts on: http://localhost:8084
