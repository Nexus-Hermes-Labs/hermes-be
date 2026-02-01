use crate::domain::user::valueobject::{CustomStatus, UserPrivacySettings, UserStatus};
use crate::domain::user::UserRole;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,

    // ─── Identity ────────────────────────────────────
    pub username: String,
    pub discriminator: String,
    pub display_name: String,

    // ─── Profile ─────────────────────────────────────
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub bio: Option<String>,

    // ─── Read-only (Presence Service owns) ──────────────
    pub status: UserStatus,
    pub custom_status: Option<CustomStatus>,

    // ─── Privacy (User Service owns) ─────────────────
    pub privacy_settings: UserPrivacySettings,

    // ─── Read-only (Auth Service owns) ───────────────
    pub role: UserRole,
    pub is_active: bool,

    // ─── Metadata ────────────────────────────────────
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}