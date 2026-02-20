//! User Profile Domain
//!
//! Handles user identity, profile information, and presence status.
//!
//! ## Aggregate Root
//! - `UserProfile` - Main aggregate containing user identity and profile data
//!
//! ## Value Objects
//! - `Username` - Unique username (3-32 chars, lowercase alphanumeric + underscore)
//! - `UserStatus` - Presence status (Online, Idle, DoNotDisturb, Offline)
//!
//! ## Business Rules
//! - Username must be unique across all users
//! - Username can only be changed once per 30 days
//! - Display name can contain unicode, spaces, emojis (non-unique)
//! - Bio maximum 500 characters
//! - Custom status text maximum 128 characters

mod entity;
mod error;
mod repository;
mod valueobject;

pub use entity::UserProfile;
pub use error::UserProfileError;
pub use repository::UserProfileRepository;
pub use valueobject::{UserStatus, Username};
