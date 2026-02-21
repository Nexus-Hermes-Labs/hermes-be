use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de, Deserialize, Deserializer};
use uuid::Uuid;

/// Only accepts the canonical UUID format: `[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}`.
///
/// Axum's default `Path<Uuid>` extractor accepts multiple formats (no hyphens, uppercase, URN,
/// braces) because the `uuid` crate's `FromStr` implementation is lenient. Using `StrictUuid`
/// in path extractors ensures that non-canonical inputs are rejected with 400 Bad Request,
/// aligning the API's runtime behaviour with the `format: uuid` OpenAPI constraint.
#[derive(Debug, Clone, Copy)]
pub struct StrictUuid(pub Uuid);

static UUID_CANONICAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
        .expect("valid UUID regex")
});

impl<'de> Deserialize<'de> for StrictUuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if !UUID_CANONICAL_RE.is_match(&s) {
            return Err(de::Error::custom(
                "invalid UUID: must be lowercase hyphenated (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)",
            ));
        }
        Uuid::parse_str(&s)
            .map(StrictUuid)
            .map_err(de::Error::custom)
    }
}
