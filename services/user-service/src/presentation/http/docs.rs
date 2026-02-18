use utoipa::OpenApi;

use crate::presentation::dto::user_profile::{request::*, response::*};
use crate::presentation::dto::user_relationship::*;
use crate::presentation::dto::user_privacy::*;
use crate::presentation::http::handlers::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        user_profile::get_profile,
        user_profile::get_profile_by_username,
        user_profile::create_profile,
        user_profile::update_profile,
        user_profile::change_username,
        user_profile::delete_profile,
        user_profile::update_status,
        user_profile::set_custom_status,
        user_profile::clear_custom_status,
        user_profile::search_users,
        user_profile::get_online_users,
        user_profile::check_username_availability,

        user_relationship::send_friend_request,
        user_relationship::accept_friend_request,
        user_relationship::decline_friend_request,
        user_relationship::remove_friend,
        user_relationship::block_user,
        user_relationship::unblock_user,
        user_relationship::get_relationship,
        user_relationship::get_friends,
        user_relationship::get_incoming_requests,
        user_relationship::get_outgoing_requests,
        user_relationship::get_blocked_users,

        user_privacy::get_privacy_settings,
        user_privacy::update_dm_privacy,
        user_privacy::update_friend_request_privacy,
        user_privacy::update_visibility,
        user_privacy::update_content_settings,
        user_privacy::apply_preset,

        user_me::get_profile,
        user_me::update_profile,
        user_me::delete_profile,
        user_me::change_username,
        user_me::update_status,
        user_me::set_custom_status,
        user_me::clear_custom_status,
        user_me::get_privacy_settings,
        user_me::update_dm_privacy,
        user_me::update_friend_request_privacy,
        user_me::update_visibility,
        user_me::update_content_settings,
        user_me::apply_preset,
        user_me::send_friend_request,
        user_me::accept_friend_request,
        user_me::decline_friend_request,
        user_me::remove_friend,
        user_me::block_user,
        user_me::unblock_user,
        user_me::get_friends,
        user_me::get_incoming_requests,
        user_me::get_outgoing_requests,
        user_me::get_blocked_users,
        user_me::get_relationship,
    ),
    components(
        schemas(
            ProfileResponse, CustomStatusResponse, ProfileListResponse, 
            UsernameAvailabilityResponse, OnlineUsersResponse,
            CreateProfileRequest, UpdateProfileRequest, ChangeUsernameRequest,
            UpdateStatusRequest, SetCustomStatusRequest, SearchUsersRequest,
            RelationshipRequest, RelationshipResponse, Pagination,
            UpdateDmPrivacyRequest, UpdateFriendRequestPrivacyRequest,
            UpdateVisibilityRequest, UpdateContentSettingsRequest,
            ApplyPresetRequest, PrivacySettingsResponse
        )
    ),
    tags(
        (name = "user-service", description = "User Profile and Relationship Management API")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
