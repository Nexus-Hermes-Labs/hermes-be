# Voice Service

**P2P WebRTC** voice communication service for Hermes (MVP).

## Responsibilities

- WebRTC signaling for P2P voice channels (2 users max)
- Voice session management
- SDP offer/answer exchange
- ICE candidate exchange
- Mute/unmute state tracking

## Important Note

This is the **MVP version** using **Peer-to-Peer WebRTC**.
- ✅ Works great for 1-on-1 calls
- ⚠️ Limited to 2 users per voice channel
- ❌ No media server (no mixing, no routing)

For production with 10+ users, consider using:
- LiveKit
- Mediasoup
- Janus Gateway

## P2P WebRTC Flow

```
User A                    Voice Service                    User B
  |                            |                             |
  |--- Join Voice ------------>|                             |
  |<-- Session Info -----------|                             |
  |                            |<--- Join Voice -------------|
  |                            |---- Session Info ---------->|
  |                            |                             |
  |--- SDP Offer ------------->|                             |
  |                            |---- Forward Offer --------->|
  |                            |<--- SDP Answer -------------|
  |<-- Forward Answer ---------|                             |
  |                            |                             |
  |<========= RTP Audio Direct Connection =================>|
```

## API Endpoints

### Join Voice Channel
```http
POST /voice/join
Authorization: Bearer <token>
Content-Type: application/json

{
  "channel_id": "uuid",
  "mute": false,
  "deaf": false
}
```

Response:
```json
{
  "session_id": "uuid",
  "ice_servers": [
    {
      "urls": ["stun:stun.l.google.com:19302"]
    }
  ],
  "peer_id": "uuid" // if another user is already in channel
}
```

### Update Voice State
```http
PATCH /voice/state
Authorization: Bearer <token>
Content-Type: application/json

{
  "session_id": "uuid",
  "mute": true,
  "deaf": false
}
```

### Leave Voice Channel
```http
POST /voice/leave
Authorization: Bearer <token>
Content-Type: application/json

{
  "session_id": "uuid"
}
```

### WebRTC Signaling
```http
POST /voice/signal
Authorization: Bearer <token>
Content-Type: application/json

{
  "session_id": "uuid",
  "type": "offer|answer|ice",
  "sdp": "v=0...",
  "candidate": { /* ICE candidate */ }
}
```

## NATS Events

### Published Events
- `voice.session.created` - User joined voice
- `voice.session.updated` - Voice state changed
- `voice.session.ended` - User left voice
- `voice.speaking.started` - User started speaking
- `voice.speaking.stopped` - User stopped speaking

### Subscribed Events
- `channel.deleted` - End all voice sessions
- `user.banned` - Disconnect user

## Environment Variables

```bash
DATABASE_URL=postgres://hermes:password@localhost:5432/hermes
REDIS_URL=redis://:password@localhost:6379
NATS_URL=nats://localhost:4222
MEDIA_SERVER_URL=http://localhost:8089
TURN_URL=turn:localhost:3478
TURN_USERNAME=hermes
TURN_PASSWORD=hermes_turn_password
PORT=8085
```

## Running

```bash
cargo run --bin voice-service
```

Server starts on: http://localhost:8085
