//! User Privacy Settings Domain
//!
//! Handles user privacy preferences and authorization checks.
//!
//! ## Why Separate Domain?
//! Privacy is separated from UserProfile because:
//! - Different update frequency (privacy changes often, profile rarely)
//! - Different access patterns (privacy for authorization, profile for display)
//! - Independent lifecycle and transactions
//! - Clear bounded context
//!
//! ## Aggregate Root
//! - `UserPrivacySettings` - Privacy preferences and visibility settings
//!
//! ## Value Objects
//! - `DmPrivacy` - Who can send DMs (Everyone, Friends, ServerMembers, None)
//! - `FriendRequestPrivacy` - Who can send friend requests
//! - `ContentFilterLevel` - Content filtering strictness (Off, Medium, Strict)
//!
//! ## Business Rules
//! - Privacy settings auto-created when user registers (DB trigger)
//! - Authorization checks done via `can_receive_dm_from()` methods
//! - Privacy presets available (Public, FriendsOnly, Private)

mod entity;
mod error;
mod repository;
mod valueobject;

pub use entity::{PrivacyPreset, Relationship, UserPrivacySettings};
pub use error::UserPrivacyError;
pub use repository::UserPrivacyRepository;
pub use valueobject::{ContentFilterLevel, DmPrivacy, FriendRequestPrivacy};
