use std::sync::Arc;

use common::proto::user::v1::user_service_server::UserService;

use crate::{
    application::services::{
        discriminator::DiscriminatorService, user::service::UserApplicationService,
        user_relationship::service::UserRelationshipApplicationService,
    },
    infrastructure::persistence::postgres::{
        user_relationship::repository::PostgresUserRelationshipRepository,
        user_repository::repository::PostgresUserRepository,
    },
};

// =====================================================
// gRPC SERVER IMPLEMENTATION
// =====================================================

pub struct UserServiceServer {
    user_service: Arc<UserApplicationService<PostgresUserRepository>>,
    discriminator_service: Arc<DiscriminatorService>,
    relationship_service: Arc<
        UserRelationshipApplicationService<
            PostgresUserRelationshipRepository,
            PostgresUserRepository,
        >,
    >,
}

impl UserServiceServer {
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
impl UserService for UserServiceServer {}
