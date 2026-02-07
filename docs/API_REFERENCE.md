# Discord Clone - API Reference

**Version:** 1.0.0-MVP  
**Last Updated:** Week 3, Day 2  
**Base URL:** `http://localhost:8080` (Development)

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [HTTP REST APIs](#http-rest-apis)
  - [Auth Service](#auth-service)
  - [User Service](#user-service)
- [gRPC APIs](#grpc-apis)
  - [User Service gRPC](#user-service-grpc)
- [Event Bus (Future)](#event-bus-future)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)

---

## Overview

The Discord Clone API uses a hybrid communication approach:
- **HTTP REST**: Client-to-service communication
- **gRPC**: Service-to-service synchronous communication
- **NATS** (Post-MVP): Service-to-service asynchronous events

### API Design Principles

1. **RESTful**: Resources are nouns, actions are HTTP verbs
2. **Versioned**: All endpoints include `/v1/` prefix
3. **Consistent**: Uniform error format across all services
4. **Idempotent**: PUT, DELETE operations are idempotent
5. **Paginated**: List endpoints support cursor-based pagination

---

## Authentication

### JWT Bearer Token

All authenticated endpoints require a JWT token in the Authorization header:
```http
Authorization: Bearer <jwt_token>
```

### Token Structure
```json
{
  "sub": "user_id",
  "username": "alice",
  "discriminator": "0042",
  "email": "alice@example.com",
  "exp": 1735689600,
  "iat": 1735603200
}
```

### Token Expiry
- **Access Token**: 15 minutes
- **Refresh Token**: 7 days

---

## HTTP REST APIs

### Auth Service

Base URL: `http://localhost:8081`

#### POST /v1/auth/register

Register a new user account.

**Request:**
```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "SecurePass123!",
  "display_name": "Alice Wonder"
}
```

**Response:** `201 Created`
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "alice",
    "discriminator": "0042",
    "email": "alice@example.com",
    "display_name": "Alice Wonder",
    "avatar_url": null,
    "created_at": "2025-01-01T00:00:00Z"
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900
}
```

**Errors:**
- `400 Bad Request`: Invalid input (weak password, invalid email)
- `409 Conflict`: Email already exists
- `503 Service Unavailable`: No available discriminators

**Validations:**
- Username: 2-32 characters, alphanumeric + underscore
- Email: Valid RFC 5322 format
- Password: Min 8 characters, 1 uppercase, 1 lowercase, 1 number
- Display name: Max 32 characters

---

#### POST /v1/auth/login

Authenticate and receive JWT tokens.

**Request:**
```json
{
  "email": "alice@example.com",
  "password": "SecurePass123!"
}
```

**Response:** `200 OK`
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "alice",
    "discriminator": "0042",
    "email": "alice@example.com",
    "display_name": "Alice Wonder",
    "avatar_url": "https://cdn.example.com/avatars/alice.png",
    "status": "online",
    "custom_status": {
      "text": "Building something cool",
      "emoji": "🚀",
      "expires_at": "2025-01-02T00:00:00Z"
    }
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900
}
```

**Errors:**
- `400 Bad Request`: Missing credentials
- `401 Unauthorized`: Invalid email or password
- `429 Too Many Requests`: Rate limit exceeded

---

#### POST /v1/auth/refresh

Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response:** `200 OK`
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900
}
```

**Errors:**
- `401 Unauthorized`: Invalid or expired refresh token

---

#### POST /v1/auth/logout

Invalidate current tokens.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Response:** `204 No Content`

---

### User Service

Base URL: `http://localhost:8082`

#### GET /v1/users/me

Get current authenticated user profile.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "discriminator": "0042",
  "email": "alice@example.com",
  "display_name": "Alice Wonder",
  "bio": "Software engineer passionate about distributed systems",
  "avatar_url": "https://cdn.example.com/avatars/alice.png",
  "banner_url": "https://cdn.example.com/banners/alice.png",
  "status": "online",
  "custom_status": {
    "text": "Building something cool",
    "emoji": "🚀",
    "expires_at": "2025-01-02T00:00:00Z"
  },
  "privacy_settings": {
    "allow_dms_from": "friends",
    "allow_friend_requests_from": "everyone",
    "show_online_status": true
  },
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T12:00:00Z"
}
```

---

#### PATCH /v1/users/me

Update current user profile.

**Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
```

**Request:**
```json
{
  "display_name": "Alice in Wonderland",
  "bio": "Exploring the rabbit hole of distributed systems",
  "avatar_url": "https://cdn.example.com/avatars/alice-new.png",
  "banner_url": "https://cdn.example.com/banners/alice-new.png"
}
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "discriminator": "0042",
  "display_name": "Alice in Wonderland",
  "bio": "Exploring the rabbit hole of distributed systems",
  "avatar_url": "https://cdn.example.com/avatars/alice-new.png",
  "banner_url": "https://cdn.example.com/banners/alice-new.png",
  "updated_at": "2025-01-01T13:00:00Z"
}
```

**Validations:**
- Display name: Max 32 characters
- Bio: Max 190 characters
- URLs: Valid HTTPS URLs

**Errors:**
- `400 Bad Request`: Validation failed
- `401 Unauthorized`: Invalid token
- `413 Payload Too Large`: Bio too long

---

#### PUT /v1/users/me/status

Set custom status (text + emoji).

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "text": "In a meeting",
  "emoji": "📅",
  "expires_at": "2025-01-01T15:00:00Z"
}
```

**Response:** `200 OK`
```json
{
  "text": "In a meeting",
  "emoji": "📅",
  "expires_at": "2025-01-01T15:00:00Z"
}
```

**Validations:**
- Text: Max 128 characters
- Emoji: Valid Unicode emoji
- Expires: Max 24 hours from now

---

#### DELETE /v1/users/me/status

Clear custom status.

**Response:** `204 No Content`

---

#### PATCH /v1/users/me/privacy

Update privacy settings.

**Request:**
```json
{
  "allow_dms_from": "friends",
  "allow_friend_requests_from": "friends_of_friends",
  "show_online_status": false
}
```

**Response:** `200 OK`
```json
{
  "allow_dms_from": "friends",
  "allow_friend_requests_from": "friends_of_friends",
  "show_online_status": false
}
```

**Enums:**
- `allow_dms_from`: `everyone` | `friends` | `server_members` | `none`
- `allow_friend_requests_from`: `everyone` | `friends_of_friends` | `none`

---

#### GET /v1/users/search

Search users by username.

**Query Parameters:**
- `q` (required): Search query (min 2 chars)
- `limit` (optional): Results per page (default: 20, max: 50)
- `cursor` (optional): Pagination cursor

**Example:**
```http
GET /v1/users/search?q=alice&limit=10
```

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "alice",
      "discriminator": "0042",
      "display_name": "Alice Wonder",
      "avatar_url": "https://cdn.example.com/avatars/alice.png",
      "status": "online"
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6IjU1MGU4NDAwLWUyOWItNDFkNC1hNzE2LTQ0NjY1NTQ0MDAwMCJ9",
    "has_more": false
  }
}
```

---

### Friend Relationships

#### POST /v1/friends/requests

Send a friend request.

**Request:**
```json
{
  "username": "bob",
  "discriminator": "1337",
  "message": "Hey! Let's be friends 👋"
}
```

**Response:** `201 Created`
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "sender_id": "550e8400-e29b-41d4-a716-446655440000",
  "receiver": {
    "id": "770e8400-e29b-41d4-a716-446655440000",
    "username": "bob",
    "discriminator": "1337",
    "display_name": "Bob Builder",
    "avatar_url": "https://cdn.example.com/avatars/bob.png"
  },
  "message": "Hey! Let's be friends 👋",
  "status": "pending",
  "created_at": "2025-01-01T14:00:00Z"
}
```

**Errors:**
- `404 Not Found`: User not found
- `409 Conflict`: Already friends / Request already sent
- `403 Forbidden`: User blocked you / Privacy settings

---

#### GET /v1/friends/requests/incoming

Get pending incoming friend requests.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 20, max: 50)

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "sender": {
        "id": "770e8400-e29b-41d4-a716-446655440000",
        "username": "bob",
        "discriminator": "1337",
        "display_name": "Bob Builder",
        "avatar_url": "https://cdn.example.com/avatars/bob.png"
      },
      "message": "Hey! Let's be friends 👋",
      "created_at": "2025-01-01T14:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 3,
    "total_pages": 1
  }
}
```

---

#### GET /v1/friends/requests/outgoing

Get pending outgoing friend requests (sent by you).

**Response:** `200 OK` (Same format as incoming)

---

#### POST /v1/friends/requests/{user_id}/accept

Accept a friend request.

**Response:** `200 OK`
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "username": "bob",
  "discriminator": "1337",
  "display_name": "Bob Builder",
  "avatar_url": "https://cdn.example.com/avatars/bob.png",
  "status": "online",
  "friend_since": "2025-01-01T14:30:00Z"
}
```

**Errors:**
- `404 Not Found`: Friend request not found
- `403 Forbidden`: Not the receiver

---

#### POST /v1/friends/requests/{user_id}/decline

Decline a friend request.

**Response:** `204 No Content`

---

#### DELETE /v1/friends/requests/{user_id}

Cancel a sent friend request.

**Response:** `204 No Content`

---

#### GET /v1/friends

Get list of friends.

**Query Parameters:**
- `page`, `page_size`: Pagination

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "username": "bob",
      "discriminator": "1337",
      "display_name": "Bob Builder",
      "avatar_url": "https://cdn.example.com/avatars/bob.png",
      "status": "online",
      "custom_status": {
        "text": "Working on projects",
        "emoji": "🔨"
      },
      "friend_since": "2025-01-01T14:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

---

#### DELETE /v1/friends/{user_id}

Remove a friend (unfriend).

**Response:** `204 No Content`

---

#### GET /v1/friends/count

Get friend count (for profile stats).

**Response:** `200 OK`
```json
{
  "count": 42
}
```

---

### Block System

#### POST /v1/blocked

Block a user.

**Request:**
```json
{
  "username": "spammer",
  "discriminator": "9999"
}
```

**Response:** `201 Created`
```json
{
  "id": "880e8400-e29b-41d4-a716-446655440000",
  "username": "spammer",
  "discriminator": "9999",
  "display_name": "Spammer",
  "blocked_at": "2025-01-01T15:00:00Z"
}
```

**Side Effects:**
- Removes any existing friendship
- Cancels pending friend requests
- User cannot send you messages
- User cannot see your online status

---

#### GET /v1/blocked

Get blocked users list.

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440000",
      "username": "spammer",
      "discriminator": "9999",
      "display_name": "Spammer",
      "blocked_at": "2025-01-01T15:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 5,
    "total_pages": 1
  }
}
```

---

#### DELETE /v1/blocked/{user_id}

Unblock a user.

**Response:** `204 No Content`

---

## gRPC APIs

### User Service gRPC

**Host:** `localhost:50051`  
**Proto File:** `proto/user_service.proto`

#### Service Definition
```protobuf
syntax = "proto3";

package user.v1;

service UserService {
  // Discriminator operations (for Auth Service)
  rpc GenerateDiscriminator(GenerateDiscriminatorRequest) 
      returns (GenerateDiscriminatorResponse);
  
  rpc CheckUsernameAvailability(CheckUsernameAvailabilityRequest) 
      returns (CheckUsernameAvailabilityResponse);
  
  // User queries (for Auth Service)
  rpc GetUserById(GetUserByIdRequest) 
      returns (GetUserByIdResponse);
  
  rpc GetUserByEmail(GetUserByEmailRequest) 
      returns (GetUserByEmailResponse);
  
  // Username update (for User Service)
  rpc UpdateUsername(UpdateUsernameRequest) 
      returns (UpdateUsernameResponse);
  
  // Friendship checks (for privacy)
  rpc AreFriends(AreFriendsRequest) 
      returns (AreFriendsResponse);
  
  rpc IsBlocked(IsBlockedRequest) 
      returns (IsBlockedResponse);
}
```

---

#### GenerateDiscriminator

Generate next available discriminator for a username.

**Request:**
```protobuf
message GenerateDiscriminatorRequest {
  string username = 1;
}
```

**Response:**
```protobuf
message GenerateDiscriminatorResponse {
  string discriminator = 1;  // e.g., "0042"
}
```

**gRPC Status Codes:**
- `OK`: Success
- `RESOURCE_EXHAUSTED`: No available discriminators (9999 limit reached)
- `INVALID_ARGUMENT`: Invalid username format

**Example (Rust):**
```rust
let response = user_client
    .generate_discriminator(GenerateDiscriminatorRequest {
        username: "alice".to_string(),
    })
    .await?;

println!("Discriminator: {}", response.into_inner().discriminator);
// Output: "0001"
```

---

#### CheckUsernameAvailability

Check if username#discriminator is available.

**Request:**
```protobuf
message CheckUsernameAvailabilityRequest {
  string username = 1;
  string discriminator = 2;
}
```

**Response:**
```protobuf
message CheckUsernameAvailabilityResponse {
  bool available = 1;
}
```

---

#### GetUserById

Retrieve user by ID (for JWT generation).

**Request:**
```protobuf
message GetUserByIdRequest {
  string user_id = 1;  // UUID as string
}
```

**Response:**
```protobuf
message GetUserByIdResponse {
  optional User user = 1;
}

message User {
  string id = 1;
  string username = 2;
  string discriminator = 3;
  string email = 4;
  string display_name = 5;
  optional string avatar_url = 6;
  optional string bio = 7;
  string status = 8;  // online, offline, idle, dnd
  optional CustomStatus custom_status = 9;
  string created_at = 10;
  string updated_at = 11;
}

message CustomStatus {
  string text = 1;
  string emoji = 2;
  optional string expires_at = 3;
}
```

**gRPC Status Codes:**
- `OK`: User found
- `NOT_FOUND`: User not found
- `INVALID_ARGUMENT`: Invalid UUID format

---

#### AreFriends

Check if two users are friends (for privacy checks).

**Request:**
```protobuf
message AreFriendsRequest {
  string user_id = 1;
  string other_user_id = 2;
}
```

**Response:**
```protobuf
message AreFriendsResponse {
  bool are_friends = 1;
}
```

---

#### IsBlocked

Check if user has blocked target (for interaction prevention).

**Request:**
```protobuf
message IsBlockedRequest {
  string blocker_id = 1;
  string blocked_id = 2;
}
```

**Response:**
```protobuf
message IsBlockedResponse {
  bool is_blocked = 1;
}
```

---

## Event Bus (Future - Post-MVP)

**Transport:** NATS JetStream  
**Subjects:** Domain-driven subject naming

### Published Events

#### user.created

Published when a new user registers.

**Subject:** `user.created`

**Payload:**
```json
{
  "event_id": "990e8400-e29b-41d4-a716-446655440000",
  "event_type": "user.created",
  "timestamp": "2025-01-01T16:00:00Z",
  "version": "1.0",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "alice",
    "discriminator": "0042",
    "email": "alice@example.com",
    "created_at": "2025-01-01T16:00:00Z"
  }
}
```

**Consumers:**
- Notification Service: Send welcome email
- Analytics Service: Track user signup
- Audit Service: Log account creation

---

#### friend_request.sent

Published when a friend request is sent.

**Subject:** `friend_request.sent`

**Payload:**
```json
{
  "event_id": "aa0e8400-e29b-41d4-a716-446655440000",
  "event_type": "friend_request.sent",
  "timestamp": "2025-01-01T16:30:00Z",
  "version": "1.0",
  "data": {
    "request_id": "bb0e8400-e29b-41d4-a716-446655440000",
    "sender_id": "550e8400-e29b-41d4-a716-446655440000",
    "receiver_id": "770e8400-e29b-41d4-a716-446655440000",
    "message": "Hey! Let's be friends 👋",
    "sent_at": "2025-01-01T16:30:00Z"
  }
}
```

**Consumers:**
- Notification Service: Send push notification
- Email Service: Send email notification
- Analytics Service: Track social graph

---

### Subscribed Events

#### user.deleted

Subscribed to handle user deletion across services.

**Subject:** `user.deleted`

**Payload:**
```json
{
  "event_id": "cc0e8400-e29b-41d4-a716-446655440000",
  "event_type": "user.deleted",
  "timestamp": "2025-01-01T17:00:00Z",
  "version": "1.0",
  "data": {
    "user_id": "880e8400-e29b-41d4-a716-446655440000",
    "deleted_at": "2025-01-01T17:00:00Z",
    "reason": "user_requested"
  }
}
```

**Actions:**
- Remove user from all friendships
- Delete pending friend requests
- Anonymize blocked relationships

---

## Error Handling

### Error Response Format

All errors follow a consistent format:
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Username must be between 2 and 32 characters",
    "details": {
      "field": "username",
      "constraint": "length",
      "min": 2,
      "max": 32
    },
    "request_id": "dd0e8400-e29b-41d4-a716-446655440000"
  }
}
```

### HTTP Status Codes

| Code | Meaning | Use Case |
|------|---------|----------|
| 200 | OK | Successful GET, PATCH |
| 201 | Created | Successful POST |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Validation error |
| 401 | Unauthorized | Missing/invalid token |
| 403 | Forbidden | No permission |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Duplicate resource |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Dependency down |

### Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Input validation failed |
| `AUTHENTICATION_REQUIRED` | No token provided |
| `INVALID_TOKEN` | Token invalid/expired |
| `PERMISSION_DENIED` | Not authorized |
| `NOT_FOUND` | Resource doesn't exist |
| `ALREADY_EXISTS` | Duplicate entry |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `INTERNAL_ERROR` | Server error |
| `SERVICE_UNAVAILABLE` | Dependency unavailable |

### gRPC Status Codes

| Status | Description |
|--------|-------------|
| `OK` | Success |
| `INVALID_ARGUMENT` | Invalid input |
| `NOT_FOUND` | Resource not found |
| `ALREADY_EXISTS` | Duplicate |
| `PERMISSION_DENIED` | No permission |
| `UNAUTHENTICATED` | Invalid credentials |
| `RESOURCE_EXHAUSTED` | Rate limit / quota |
| `UNAVAILABLE` | Service down |
| `INTERNAL` | Server error |

---

## Rate Limiting

### HTTP Endpoints

**Global Limits:**
- Unauthenticated: 100 requests/minute/IP
- Authenticated: 1000 requests/minute/user

**Endpoint-Specific:**
- `/auth/login`: 5 requests/minute/IP
- `/auth/register`: 3 requests/hour/IP
- `/friends/requests`: 10 requests/minute/user

**Headers:**
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1735690800
```

**Error Response:**
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Try again in 42 seconds.",
    "details": {
      "retry_after": 42
    }
  }
}
```

---

## Pagination

### Cursor-Based Pagination

**Request:**
```http
GET /v1/users/search?q=alice&limit=20&cursor=eyJpZCI6IjEyMyJ9
```

**Response:**
```json
{
  "items": [...],
  "pagination": {
    "next_cursor": "eyJpZCI6IjE0MyJ9",
    "has_more": true
  }
}
```

### Offset-Based Pagination

**Request:**
```http
GET /v1/friends?page=2&page_size=20
```

**Response:**
```json
{
  "items": [...],
  "pagination": {
    "page": 2,
    "page_size": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

---

## Versioning

### API Versioning Strategy

- **URL-based versioning**: `/v1/`, `/v2/`
- **Proto versioning**: `user.v1`, `user.v2`
- **Backward compatibility**: Maintain v1 for 6 months after v2 release

### Deprecation Policy

1. Announce deprecation 3 months in advance
2. Add deprecation headers:
```http
   Deprecation: true
   Sunset: Wed, 01 Apr 2026 00:00:00 GMT
   Link: </v2/users/me>; rel="successor-version"
```
3. Remove deprecated version after sunset date

---

**End of API Reference**