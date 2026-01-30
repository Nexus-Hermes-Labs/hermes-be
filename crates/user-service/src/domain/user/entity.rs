use crate::domain::user::valueobject::UserStatus;
use crate::domain::user::UserRole;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// User domain
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,

    // ================= IDENTITY =================
    pub username: String,
    pub discriminator: String,  // "0000" - "9999"
    pub display_name: String,

    // ================= PROFILE =================
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,    // ✅ NEW
    pub bio: Option<String>,

    // ================= STATUS =================
    pub status: UserStatus,
    pub custom_status: Option<CustomStatus>,  // ✅ NEW (value object)

    // ================= PRIVACY =================
    pub privacy_settings: UserPrivacySettings,  // ✅ NEW (value object)

    // ================= METADATA =================
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
