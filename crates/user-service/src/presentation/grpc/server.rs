use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;
use common::proto::user::v1::user_service_server;
use super::user_service::{UserGrpcService};
use common::proto::user::v1::user_service_server::{UserService as UserServiceTrait, UserServiceServer};
use crate::application::services::discriminator::DiscriminatorService;
use crate::application::services::user::service::UserApplicationService;
use crate::application::services::user_relationship::service::UserRelationshipApplicationService;
use crate::domain::user::UserRepository;
use crate::domain::user_relationship::repository::UserRelationshipRepository;
use crate::infrastructure::persistence::postgres::user_relationship::repository::PostgresUserRelationshipRepository;
use crate::infrastructure::persistence::postgres::user_repository::repository::PostgresUserRepository;

pub async fn start_grpc_server(
    port: u16,
    user_service: Arc<UserApplicationService<PostgresUserRepository>>,
    discriminator_service: Arc<DiscriminatorService>,
    relationship_service: Arc<
        UserRelationshipApplicationService<
            PostgresUserRelationshipRepository,
            PostgresUserRepository,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", port).parse()?;

    let grpc_service =
        UserGrpcService::new(user_service, discriminator_service, relationship_service);

    info!("🚀 User Service gRPC server starting on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
