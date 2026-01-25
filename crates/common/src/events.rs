use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    // Auth Events
    UserCreated(UserCreatedEvent),
    UserDeleted(UserDeletedEvent),

    // User Events
    UserProfileUpdated(UserProfileUpdatedEvent),
    FriendAdded(FriendEvent),
    FriendRemoved(FriendEvent),
    UserBlocked(UserBlockedEvent),

    // Server Events
    ServerCreated(ServerEvent),
    ServerUpdated(ServerEvent),
    ServerDeleted(ServerDeletedEvent),

    // Channel Events
    ChannelCreated(ChannelEvent),
    ChannelUpdated(ChannelEvent),
    ChannelDeleted(ChannelDeletedEvent),

    // Message Events
    MessageCreated(MessageEvent),
    MessageUpdated(MessageEvent),
    MessageDeleted(MessageDeletedEvent),

    // Voice Events
    VoiceSessionCreated(VoiceSessionEvent),
    VoiceSessionUpdated(VoiceSessionEvent),
    VoiceSessionEnded(VoiceSessionEndedEvent),
    VoiceSpeakingStarted(VoiceSpeakingEvent),
    VoiceSpeakingStopped(VoiceSpeakingEvent),

    // Stream Events
    StreamStarted(StreamEvent),
    StreamStopped(StreamStoppedEvent),
    StreamViewerJoined(StreamViewerEvent),
    StreamViewerLeft(StreamViewerEvent),

    // Presence Events
    PresenceStatusChanged(PresenceEvent),
    PresenceTypingStarted(TypingEvent),
}

// Auth Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// User Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdatedEvent {
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendEvent {
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBlockedEvent {
    pub user_id: Uuid,
    pub blocked_user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Server Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEvent {
    pub server_id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDeletedEvent {
    pub server_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Channel Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEvent {
    pub channel_id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeletedEvent {
    pub channel_id: Uuid,
    pub server_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Message Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeletedEvent {
    pub message_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

// Voice Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSessionEvent {
    pub session_id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub muted: bool,
    pub deafened: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSessionEndedEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSpeakingEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Stream Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub stream_id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStoppedEvent {
    pub stream_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamViewerEvent {
    pub stream_id: Uuid,
    pub viewer_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Presence Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub user_id: Uuid,
    pub status: String,
    pub custom_status: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingEvent {
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn topic(&self) -> String {
        match self {
            Event::UserCreated(_) => "auth.user.created",
            Event::UserDeleted(_) => "auth.user.deleted",
            Event::UserProfileUpdated(_) => "user.profile.updated",
            Event::FriendAdded(_) => "user.friend.added",
            Event::FriendRemoved(_) => "user.friend.removed",
            Event::UserBlocked(_) => "user.blocked",
            Event::ServerCreated(_) => "server.created",
            Event::ServerUpdated(_) => "server.updated",
            Event::ServerDeleted(_) => "server.deleted",
            Event::ChannelCreated(_) => "channel.created",
            Event::ChannelUpdated(_) => "channel.updated",
            Event::ChannelDeleted(_) => "channel.deleted",
            Event::MessageCreated(_) => "message.created",
            Event::MessageUpdated(_) => "message.updated",
            Event::MessageDeleted(_) => "message.deleted",
            Event::VoiceSessionCreated(_) => "voice.session.created",
            Event::VoiceSessionUpdated(_) => "voice.session.updated",
            Event::VoiceSessionEnded(_) => "voice.session.ended",
            Event::VoiceSpeakingStarted(_) => "voice.speaking.started",
            Event::VoiceSpeakingStopped(_) => "voice.speaking.stopped",
            Event::StreamStarted(_) => "stream.started",
            Event::StreamStopped(_) => "stream.stopped",
            Event::StreamViewerJoined(_) => "stream.viewer.joined",
            Event::StreamViewerLeft(_) => "stream.viewer.left",
            Event::PresenceStatusChanged(_) => "presence.status.changed",
            Event::PresenceTypingStarted(_) => "presence.typing.started",
        }
        .to_string()
    }
}
