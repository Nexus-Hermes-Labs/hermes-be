use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Request body for creating a new guild.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGuildRequest {
    /// Guild display name (2–100 characters).
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    /// Optional description shown on the guild profile (max 1 000 characters).
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// URL of the guild icon image (must begin with `http://` or `https://`).
    pub icon_url: Option<String>,
}

/// Request body for updating guild settings.
///
/// All fields are optional; only the provided fields are changed.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateGuildRequest {
    /// New guild display name (2–100 characters).
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    /// New guild description (max 1 000 characters).
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// New icon URL (must begin with `http://` or `https://`).
    pub icon_url: Option<String>,
    /// New banner URL (must begin with `http://` or `https://`).
    pub banner_url: Option<String>,
    /// Visibility setting: `"public"` or `"private"`.
    pub visibility: Option<String>,
}

/// Query parameters for searching public guilds.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SearchGuildsRequest {
    /// Full-text search term matched against guild names.
    pub query: String,
    /// Maximum number of results to return (default: 20).
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip for pagination (default: 0).
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}
