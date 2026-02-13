use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::services::{UserPrivacyService, UserProfileService};
use crate::domain::user_privacy::UserPrivacyRepository;
use crate::domain::user_profile::UserProfileRepository;

// Import generated protobuf types
// use proto::user::v1::user_service_server::UserService;
// use proto::user::v1::*;

/// gRPC server implementation for UserService
///
/// This is used for service-to-service communication (e.g., auth-service calling user-service)
pub struct UserServiceGrpc<PR, VR>
where
    PR: UserProfileRepository,
    VR: UserPrivacyRepository,
{
    profile_service: Arc<UserProfileService<PR>>,
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

// Example implementation (commented out until proto is generated)
/*
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
        
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;
        
        let profile = self.profile_service
            .create_profile(user_id, req.username, req.display_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(UserProfileResponse {
            user_id: profile.id().to_string(),
            username: profile.username().as_str().to_string(),
            display_name: profile.display_name().to_string(),
            avatar_url: profile.avatar_url().map(String::from).unwrap_or_default(),
            bio: profile.bio().map(String::from).unwrap_or_default(),
            created_at: Some(prost_types::Timestamp {
                seconds: profile.created_at().timestamp(),
                nanos: profile.created_at().timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: profile.updated_at().timestamp(),
                nanos: profile.updated_at().timestamp_subsec_nanos() as i32,
            }),
        }))
    }
    
    async fn get_user_profile(
        &self,
        request: Request<GetUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();
        
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;
        
        let profile = self.profile_service
            .get_profile(user_id)
            .await
            .map_err(|e| match e {
                UserProfileServiceError::ProfileNotFound => Status::not_found("Profile not found"),
                _ => Status::internal(e.to_string()),
            })?;
        
        Ok(Response::new(UserProfileResponse {
            user_id: profile.id().to_string(),
            username: profile.username().as_str().to_string(),
            display_name: profile.display_name().to_string(),
            avatar_url: profile.avatar_url().map(String::from).unwrap_or_default(),
            bio: profile.bio().map(String::from).unwrap_or_default(),
            created_at: Some(prost_types::Timestamp {
                seconds: profile.created_at().timestamp(),
                nanos: profile.created_at().timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: profile.updated_at().timestamp(),
                nanos: profile.updated_at().timestamp_subsec_nanos() as i32,
            }),
        }))
    }
    
    // ... other gRPC methods ...
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_server_creation() {
        // Mock services would be created here
        // let profile_service = Arc::new(UserProfileService::new(...));
        // let privacy_service = Arc::new(UserPrivacyService::new(...));
        // let grpc_server = UserServiceGrpc::new(profile_service, privacy_service);
    }
}
