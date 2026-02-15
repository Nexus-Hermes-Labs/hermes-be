use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::services::{UserPrivacyService, UserProfileService};
use crate::domain::user_privacy::UserPrivacyRepository;
use crate::domain::user_profile::UserProfileRepository;
use crate::presentation::grpc::proto::user::v1::user_service_server::UserService;
use crate::presentation::grpc::proto::user::v1::{
    BatchGetUserProfilesRequest, BatchGetUserProfilesResponse, CreateUserProfileRequest,
    DeleteUserProfileRequest, GetUserProfileRequest, SearchUsersRequest, SearchUsersResponse,
    UpdateUserProfileRequest, UserProfileResponse,
};

/// gRPC server implementation for UserService
///
/// This is used for service-to-service communication (e.g., auth-service calling user-service)
pub struct UserServiceGrpc<PR, VR>
where
    PR: UserProfileRepository,
    VR: UserPrivacyRepository,
{
    profile_service: Arc<UserProfileService<PR>>,
    #[allow(dead_code)]
    privacy_service: Arc<UserPrivacyService<VR>>,
}

impl<PR, VR> UserServiceGrpc<PR, VR>
where
    PR: UserProfileRepository,
    VR: UserPrivacyRepository,
{
    pub fn new(
        profile_service: Arc<UserProfileService<PR>>,
        privacy_service: Arc<UserPrivacyService<VR>>,
    ) -> Self {
        Self {
            profile_service,
            privacy_service,
        }
    }
}

/// Convert a domain `UserProfile` to a proto `UserProfileResponse`
fn to_proto_response(profile: &crate::domain::user_profile::UserProfile) -> UserProfileResponse {
    UserProfileResponse {
        user_id: profile.id().to_string(),
        username: profile.username().to_string(),
        display_name: profile.display_name().to_string(),
        avatar_url: profile.avatar_url().unwrap_or_default().to_string(),
        bio: profile.bio().unwrap_or_default().to_string(),
        created_at: Some(prost_types::Timestamp {
            seconds: profile.created_at().timestamp(),
            nanos: profile.created_at().timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: profile.updated_at().timestamp(),
            nanos: profile.updated_at().timestamp_subsec_nanos() as i32,
        }),
    }
}

#[tonic::async_trait]
impl<PR, VR> UserService for UserServiceGrpc<PR, VR>
where
    PR: UserProfileRepository + Send + Sync + 'static,
    VR: UserPrivacyRepository + Send + Sync + 'static,
{
    async fn create_user_profile(
        &self,
        request: Request<CreateUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();

        let profile = self
            .profile_service
            .create_profile(req.username, req.display_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(to_proto_response(&profile)))
    }

    async fn get_user_profile(
        &self,
        request: Request<GetUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();

        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        let profile = self
            .profile_service
            .get_profile(user_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(to_proto_response(&profile)))
    }

    async fn update_user_profile(
        &self,
        request: Request<UpdateUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();

        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        let profile = self
            .profile_service
            .update_profile(
                user_id,
                req.display_name,
                req.bio,
                req.avatar_url,
                None, // banner_url not in proto
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(to_proto_response(&profile)))
    }

    async fn delete_user_profile(
        &self,
        request: Request<DeleteUserProfileRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        self.profile_service
            .delete_profile(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn batch_get_user_profiles(
        &self,
        request: Request<BatchGetUserProfilesRequest>,
    ) -> Result<Response<BatchGetUserProfilesResponse>, Status> {
        let req = request.into_inner();

        if req.user_ids.len() > 100 {
            return Err(Status::invalid_argument(
                "Maximum 100 user IDs per batch request",
            ));
        }

        let user_ids: Vec<Uuid> = req
            .user_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more invalid user_id formats"))?;

        let profiles = self
            .profile_service
            .get_profiles_by_ids(user_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_profiles: Vec<UserProfileResponse> =
            profiles.iter().map(to_proto_response).collect();

        Ok(Response::new(BatchGetUserProfilesResponse {
            profiles: proto_profiles,
        }))
    }

    async fn search_users(
        &self,
        request: Request<SearchUsersRequest>,
    ) -> Result<Response<SearchUsersResponse>, Status> {
        let req = request.into_inner();

        let (limit, offset) = if let Some(pagination) = &req.pagination {
            let page = pagination.page.max(1);
            let page_size = pagination.page_size.clamp(1, 100);
            (i64::from(page_size), i64::from((page - 1) * page_size))
        } else {
            (20_i64, 0_i64)
        };

        let profiles = self
            .profile_service
            .search_users(req.query, limit, offset)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_profiles: Vec<UserProfileResponse> =
            profiles.iter().map(to_proto_response).collect();

        // Build pagination response
        let pagination_response = req.pagination.map(|p| {
            let page_size = p.page_size.clamp(1, 100);
            crate::presentation::grpc::proto::common::v1::PaginationResponse {
                total_items: proto_profiles.len() as i32, // Simplified; ideally count from DB
                total_pages: 1, // Simplified
                current_page: p.page.max(1),
                page_size,
            }
        });

        Ok(Response::new(SearchUsersResponse {
            profiles: proto_profiles,
            pagination: pagination_response,
        }))
    }
}
