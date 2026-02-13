mod user_privacy;
mod user_profile;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::application::services::{UserPrivacyService, UserProfileService};
use crate::domain::user_privacy::UserPrivacyRepository;
use crate::domain::user_profile::UserProfileRepository;

use super::handlers::{UserPrivacyHandler, UserProfileHandler};

