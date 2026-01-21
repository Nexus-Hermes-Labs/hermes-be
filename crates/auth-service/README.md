# Auth Service

JWT-based authentication service for Hermes.

## Responsibilities

- User registration
- User login/logout
- JWT token generation and validation
- Password hashing with Argon2
- Refresh token management
- Session management

## API Endpoints

### Register
```http
POST /register
Content-Type: application/json

{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePassword123!"
}
```

### Login
```http
POST /login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "SecurePassword123!"
}
```

Response:
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "username": "john_doe",
    "email": "john@example.com"
  }
}
```

### Refresh Token
```http
POST /refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGc..."
}
```

### Logout
```http
POST /logout
Authorization: Bearer <access_token>
```

## Environment Variables

```bash
DATABASE_URL=postgres://hermes:password@localhost:5432/hermes
REDIS_URL=redis://:password@localhost:6379
JWT_SECRET=your-secret-key
JWT_ACCESS_EXPIRY=3600
JWT_REFRESH_EXPIRY=604800
PORT=8081
```

## Running

```bash
cargo run --bin auth-service
```

Server starts on: http://localhost:8081
