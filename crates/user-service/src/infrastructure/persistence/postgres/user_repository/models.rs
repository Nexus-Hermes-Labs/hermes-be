use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::user::entity::User;
use crate::domain::user::error::UserDomainError;
use crate::domain::user::UserRole;

use crate::domain::user::valueobject::{CustomStatus, DmPrivacy, FriendRequestPrivacy, UserPrivacySettings, UserStatus};

/// Flat database row — mirrors the columns User Service SELECTs.
///
/// Auth-owned columns (email, password_hash, email_verified,
/// email_verification_token) are intentionally absent.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    // Identity
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,

    // Profile
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub bio: Option<String>,

    // Status (Presence Service owns)
    pub status: String,
    pub custom_status_text: Option<String>,
    pub custom_status_emoji: Option<String>,
    pub custom_status_expires_at: Option<DateTime<Utc>>,

    // Privacy
    pub allow_dms_from: String,
    pub allow_friend_requests_from: String,
    pub show_online_status: bool,

    // Read-only from Auth Service
    pub role: String,
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert flat DB row → nested domain model.
/// Returns Err when an enum column contains an unrecognised value
/// (should never happen if the DB enum and Rust enum stay in sync).
impl TryFrom<UserRow> for User {
    type Error = UserDomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        // ── custom_status: only construct when at least one field is set ──
        let custom_status = match (&row.custom_status_text, &row.custom_status_emoji) {
            (None, None) => None,
            // Data came from DB so it was valid when written;
            // bypass the "expiration must be in the future" check
            // by constructing directly instead of calling ::new().
            _ => Some(CustomStatus {
                text: row.custom_status_text,
                emoji: row.custom_status_emoji,
                expires_at: row.custom_status_expires_at,
            }),
        };

        Ok(User {
            id: row.id,
            username: row.username,
            discriminator: row.discriminator,
            display_name: row.display_name.unwrap_or_default(),
            avatar_url: row.avatar_url,
            banner_url: row.banner_url,
            bio: row.bio,
            status: UserStatus::from_str(&row.status)?,
            custom_status,
            privacy_settings: UserPrivacySettings {
                allow_dms_from: DmPrivacy::from_str(&row.allow_dms_from)?,
                allow_friend_requests_from: FriendRequestPrivacy::from_str(
                    &row.allow_friend_requests_from,
                )?,
                show_online_status: row.show_online_status,
            },
            role: UserRole::from_str(&row.role)?,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}