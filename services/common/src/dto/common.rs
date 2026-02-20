use validator::Validate;

// =====================================================
// SEARCH
// =====================================================

use serde::Deserialize;

/// Search query parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SearchQuery {
    #[validate(length(min = 1, max = 100))]
    pub q: String,

    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}
