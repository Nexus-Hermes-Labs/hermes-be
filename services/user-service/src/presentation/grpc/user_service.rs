use std::sync::Arc;

use axum::http::Response;
use common::proto::user::v1::{user_service_server::UserService as UserServiceTrait, AreFriendsRequest, AreFriendsResponse, CheckUsernameAvailabilityRequest, CheckUsernameAvailabilityResponse, CustomStatus as ProtoCustomStatus, GenerateDiscriminatorRequest, GenerateDiscriminatorResponse, GetUserByEmailRequest, GetUserByEmailResponse, GetUserByIdRequest, GetUserByIdResponse, IsBlockedRequest, IsBlockedResponse, PrivacySettings as ProtoPrivacySettings, User as ProtoUser};
use tonic::{Request, Status};
use tracing::{error, info, instrument, warn};

use crate::{
    application::services::{
        discriminator::DiscriminatorService,
        user::{error::UserApplicationError, service::UserApplicationService},
        user_relationship::service::UserRelationshipApplicationService,
    },
    domain::user_profile::User,
    infrastructure::persistence::postgres::{
        user_relationship::repository::PostgresUserRelationshipRepository,
        user_profile_repository::repository::PostgresUserRepository,
    },
};

// =====================================================
// gRPC SERVER IMPLEMENTATION
// =====================================================

pub struct UserGrpcService {
    user_service: Arc<UserApplicationService<PostgresUserRepository>>,
    discriminator_service: Arc<DiscriminatorService>,
    relationship_service: Arc<
        UserRelationshipApplicationService<
            PostgresUserRelationshipRepository,
            PostgresUserRepository,
        >,
    >,
}

impl UserGrpcService {
    pub fn new(
        user_service: Arc<UserApplicationService<PostgresUserRepository>>,
        discriminator_service: Arc<DiscriminatorService>,
        relationship_service: Arc<
            UserRelationshipApplicationService<
                PostgresUserRelationshipRepository,
                PostgresUserRepository,
            >,
        >,
    ) -> Self {
        Self {
            user_service,
            discriminator_service,
            relationship_service,
        }
    }
}


#[tonic::async_trait]
impl UserServiceTrait for UserGrpcService {
    // =====================================================
    // DISCRIMINATOR OPERATIONS
    // =====================================================

    #[instrument(skip(self), fields(username = %request.get_ref().username))]
    async fn generate_discriminator(
        &self,
        request: Request<GenerateDiscriminatorRequest>,
    ) -> Result<tonic::Response<GenerateDiscriminatorResponse>, Status> {
        let req = request.into_inner();

        info!(
            "gRPC: Generating discriminator for username: {}",
            req.username
        );

        let discriminator = self
            .discriminator_service
            .generate_discriminator(&req.username)
            .await
            .map_err(|e| {
                error!("Failed to generate discriminator: {}", e);
                Self::map_application_error(e)
            })?;

        info!("gRPC: Generated discriminator: {}", discriminator);

        Ok(tonic::Response::new(GenerateDiscriminatorResponse {
            discriminator,
        }))
    }

    async fn check_username_availability(&self, request: Request<CheckUsernameAvailabilityRequest>) -> Result<tonic::Response<CheckUsernameAvailabilityResponse>, Status> {
        todo!()
    }

    async fn get_user_by_id(&self, request: Request<GetUserByIdRequest>) -> Result<tonic::Response<GetUserByIdResponse>, Status> {
        todo!()
    }

    async fn get_user_by_email(&self, request: Request<GetUserByEmailRequest>) -> Result<tonic::Response<GetUserByEmailResponse>, Status> {
        todo!()
    }

    async fn are_friends(&self, request: Request<AreFriendsRequest>) -> Result<tonic::Response<AreFriendsResponse>, Status> {
        todo!()
    }

    async fn is_blocked(&self, request: Request<IsBlockedRequest>) -> Result<tonic::Response<IsBlockedResponse>, Status> {
        todo!()
    }
}

// =====================================================
// HELPER METHODS
// =====================================================

impl UserGrpcService {
    /// Map application error to gRPC status
    fn map_application_error(error: UserApplicationError) -> Status {
        match error {
            // UserApplicationError::UserNotFound(_) => Status::not_found("User not found"),
            UserApplicationError::NoAvailableDiscriminators(_) => {
                Status::resource_exhausted("No available discriminators for this username")
            }
            // UserApplicationError::ValidationError(msg) => Status::invalid_argument(msg),
            _ => Status::internal("Internal server error"),
        }
    }

    /// Convert domain User to proto User
    fn user_to_proto(user: &User) -> ProtoUser {
        ProtoUser {
            id: user.id.to_string(),
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            banner_url: user.banner_url.clone(),
            status: user.status.as_str().to_string(),
            custom_status: user.custom_status.as_ref().map(|cs| ProtoCustomStatus {
                text: cs.text.clone(),
                emoji: cs.emoji.clone(),
                expires_at: cs.expires_at.map(|dt| dt.to_rfc3339()),
            }),
            privacy_settings: Some(ProtoPrivacySettings {
                allow_dms_from: user.privacy_settings.allow_dms_from.as_str().to_string(),
                allow_friend_requests_from: user
                    .privacy_settings
                    .allow_friend_requests_from
                    .as_str()
                    .to_string(),
                show_online_status: user.privacy_settings.show_online_status,
            }),
            role: user.role.as_str().to_string(),
            is_active: user.is_active,
            email_verified: false, // TODO: will delete in protoUser (seperation of user_profile and auth entites)
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

