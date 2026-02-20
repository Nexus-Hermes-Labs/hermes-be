use auth_service::application::services::authentication::service::AuthService;
use auth_service::infrastructure::grpc::UserGrpcClient;
use auth_service::infrastructure::persistence::postgres::{
    PostgresAuthCredentialRepository, PostgresAuthSessionRepository,
};
use auth_service::infrastructure::security::password::argon2_service::Argon2PasswordService;
use auth_service::infrastructure::security::token::sha256_service::Sha256TokenHasher;
use auth_service::presentation::grpc::proto::user::v1::user_service_server::{
    UserService, UserServiceServer,
};
use auth_service::presentation::grpc::proto::user::v1::{
    BatchGetUserProfilesRequest, BatchGetUserProfilesResponse, CreateUserProfileRequest,
    DeleteUserProfileRequest, GetUserProfileRequest, SearchUsersRequest, SearchUsersResponse,
    UpdateUserProfileRequest, UserProfileResponse,
};
use auth_service::state::app_state::AppState;
use auth_service::state::auth_state::AuthState;
use auth_service::state::shared_state::SharedState;
use axum::Router;
use common::infrastructure::messaging::NatsEventPublisher;
use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::{HealthCheck, Metrics};
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;
use tonic::{Request, Response, Status};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// User-service migrations (auth-service depends on user schema for mock gRPC)
const USER_ENUMS_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135148_create_enums.sql");
const USER_PROFILES_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135149_create_user_profiles.sql");
const USER_PRIVACY_SQL: &str = include_str!(
    "../../../user-service/migrations/20260121135150_create_user_privacy_settings.sql"
);
const USER_BADGES_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135151_create_user_badges.sql");
const USER_RELATIONSHIPS_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135152_create_user_relationships.sql");
const USER_INDEXES_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135153_create_indexes.sql");
const USER_FUNCTIONS_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135154_create_functions.sql");
const USER_TRIGGERS_SQL: &str =
    include_str!("../../../user-service/migrations/20260121135155_create_triggers.sql");

// Auth-service migrations
const AUTH_ENUMS_SQL: &str = include_str!("../../migrations/20260122000000_create_enums.sql");
const AUTH_CREDENTIALS_SQL: &str =
    include_str!("../../migrations/20260122000001_create_auth_credentials.sql");
const AUTH_SESSIONS_SQL: &str =
    include_str!("../../migrations/20260122000002_create_auth_sessions.sql");
const AUTH_AUDIT_LOG_SQL: &str =
    include_str!("../../migrations/20260122000003_create_auth_audit_log.sql");
const AUTH_INDEXES_SQL: &str = include_str!("../../migrations/20260122000004_create_indexes.sql");
const AUTH_FUNCTIONS_SQL: &str =
    include_str!("../../migrations/20260122000005_create_functions.sql");
const AUTH_TRIGGERS_SQL: &str = include_str!("../../migrations/20260122000006_create_triggers.sql");

// JWT secrets (test-only, 32+ chars)
const TEST_ACCESS_SECRET: &str = "test_access_secret_for_integration_tests_min_32_chars";
const TEST_REFRESH_SECRET: &str = "test_refresh_secret_for_integration_tests_min_32_chars";

/// Initialize the global metrics recorder exactly once across all tests.
static METRICS_INIT: std::sync::Once = std::sync::Once::new();
static mut METRICS: Option<Metrics> = None;

fn get_or_init_metrics() -> Metrics {
    METRICS_INIT.call_once(|| {
        let m = Metrics::init().expect("init metrics");
        // SAFETY: only written once inside call_once, read after
        unsafe { METRICS = Some(m) };
    });
    // SAFETY: guaranteed to be initialized after call_once
    #[allow(static_mut_refs)]
    unsafe {
        METRICS.clone().expect("metrics initialized")
    }
}

// ============================================
// MOCK EMAIL SERVICE
// ============================================

#[derive(Debug, Clone)]
pub struct MockEmailService;

#[async_trait::async_trait]
impl auth_service::domain::auth_credential::EmailService for MockEmailService {
    async fn send_verification_email(
        &self,
        to: &str,
        token: &str,
    ) -> Result<(), auth_service::application::services::authentication::error::AuthApplicationError>
    {
        tracing::info!(
            "MockEmailService: Sending verification email to {} with token {}",
            to,
            token
        );
        Ok(())
    }
}

// ============================================
// MOCK gRPC USER SERVICE
// ============================================

#[derive(Debug)]
pub struct MockUserService;

#[tonic::async_trait]
impl UserService for MockUserService {
    async fn create_user_profile(
        &self,
        request: Request<CreateUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();
        let user_id = uuid::Uuid::new_v4();
        let now = prost_types::Timestamp {
            seconds: chrono::Utc::now().timestamp(),
            nanos: 0,
        };

        Ok(Response::new(UserProfileResponse {
            user_id: user_id.to_string(),
            username: req.username,
            display_name: req.display_name,
            avatar_url: String::new(),
            bio: String::new(),
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }))
    }

    async fn get_user_profile(
        &self,
        request: Request<GetUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        let req = request.into_inner();
        let now = prost_types::Timestamp {
            seconds: chrono::Utc::now().timestamp(),
            nanos: 0,
        };

        Ok(Response::new(UserProfileResponse {
            user_id: req.user_id,
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: String::new(),
            bio: String::new(),
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }))
    }

    async fn update_user_profile(
        &self,
        _request: Request<UpdateUserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        Err(Status::unimplemented("not needed for auth tests"))
    }

    async fn delete_user_profile(
        &self,
        _request: Request<DeleteUserProfileRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("not needed for auth tests"))
    }

    async fn batch_get_user_profiles(
        &self,
        _request: Request<BatchGetUserProfilesRequest>,
    ) -> Result<Response<BatchGetUserProfilesResponse>, Status> {
        Err(Status::unimplemented("not needed for auth tests"))
    }

    async fn search_users(
        &self,
        _request: Request<SearchUsersRequest>,
    ) -> Result<Response<SearchUsersResponse>, Status> {
        Err(Status::unimplemented("not needed for auth tests"))
    }
}

/// Start a mock gRPC server on a random port and return the address.
async fn start_mock_grpc_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock gRPC listener");
    let addr = listener.local_addr().expect("get local addr");
    let addr_str = format!("http://127.0.0.1:{}", addr.port());

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(UserServiceServer::new(MockUserService))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .expect("mock gRPC server failed");
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr_str
}

// ============================================
// TEST HARNESS
// ============================================

/// Holds all testcontainers and the fully-built router.
///
/// Containers are dropped (and stopped) when `TestHarness` is dropped.
pub struct TestHarness {
    pub router: Router,
    pub pool: PgPool,
    // Keep containers alive for the test duration
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<GenericImage>,
    _nats_container: ContainerAsync<GenericImage>,
}

impl TestHarness {
    pub async fn new() -> Self {
        // 1. Start PostgreSQL
        let pg_container = Postgres::default()
            .start()
            .await
            .expect("start postgres container");
        let pg_host = pg_container.get_host().await.expect("get pg host");
        let pg_port = pg_container
            .get_host_port_ipv4(5432)
            .await
            .expect("get pg port");
        let pg_url = format!("postgres://postgres:postgres@{pg_host}:{pg_port}/postgres");

        let pool = PgPool::connect(&pg_url)
            .await
            .expect("connect to test postgres");

        // Run migrations in dependency order
        let migrations: &[(&str, &str)] = &[
            ("user enums", USER_ENUMS_SQL),
            ("user profiles", USER_PROFILES_SQL),
            ("user privacy", USER_PRIVACY_SQL),
            ("user badges", USER_BADGES_SQL),
            ("user relationships", USER_RELATIONSHIPS_SQL),
            ("user indexes", USER_INDEXES_SQL),
            ("user functions", USER_FUNCTIONS_SQL),
            ("user triggers", USER_TRIGGERS_SQL),
            ("auth enums", AUTH_ENUMS_SQL),
            ("auth credentials", AUTH_CREDENTIALS_SQL),
            ("auth sessions", AUTH_SESSIONS_SQL),
            ("auth audit log", AUTH_AUDIT_LOG_SQL),
            ("auth indexes", AUTH_INDEXES_SQL),
            ("auth functions", AUTH_FUNCTIONS_SQL),
            ("auth triggers", AUTH_TRIGGERS_SQL),
        ];
        for (name, sql) in migrations {
            sqlx::raw_sql(sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("run migration '{}': {}", name, e));
        }

        // 2. Start Redis
        let redis_container = GenericImage::new("redis", "7-alpine")
            .with_exposed_port(6379.into())
            .with_wait_for(testcontainers::core::WaitFor::message_on_stdout(
                "Ready to accept connections",
            ))
            .start()
            .await
            .expect("start redis container");
        let redis_host = redis_container.get_host().await.expect("get redis host");
        let redis_port = redis_container
            .get_host_port_ipv4(6379)
            .await
            .expect("get redis port");
        let redis_url = format!("redis://{}:{}", redis_host, redis_port);
        let redis_client = redis::Client::open(redis_url.as_str()).expect("create redis client");
        let redis_conn = redis::aio::ConnectionManager::new(redis_client)
            .await
            .expect("create redis connection manager");

        // 3. Start NATS
        let nats_container = GenericImage::new("nats", "2-alpine")
            .with_exposed_port(4222.into())
            .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
                "Server is ready",
            ))
            .start()
            .await
            .expect("start nats container");
        let nats_host = nats_container.get_host().await.expect("get nats host");
        let nats_port = nats_container
            .get_host_port_ipv4(4222)
            .await
            .expect("get nats port");
        let nats_url = format!("nats://{}:{}", nats_host, nats_port);

        // Retry NATS connection — container may need extra time to accept connections
        let mut event_publisher = None;
        for attempt in 0..10 {
            match NatsEventPublisher::new("test-auth-service", &nats_url).await {
                Ok(ep) => {
                    event_publisher = Some(ep);
                    break;
                }
                Err(_) if attempt < 9 => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(e) => panic!("connect to nats after retries: {e}"),
            }
        }
        let event_publisher = Arc::new(event_publisher.unwrap());

        // 4. Start mock gRPC server
        let grpc_addr = start_mock_grpc_server().await;
        let user_profile_client = Arc::new(
            UserGrpcClient::new(grpc_addr)
                .await
                .expect("create grpc client"),
        );

        // 5. Build services
        let jwt_manager = Arc::new(
            JwtManager::new("test-auth-service", TEST_ACCESS_SECRET, TEST_REFRESH_SECRET)
                .expect("create jwt manager"),
        );
        let credential_repo = Arc::new(PostgresAuthCredentialRepository::new(pool.clone()));
        let session_repo = Arc::new(PostgresAuthSessionRepository::new(pool.clone()));
        let password_service = Arc::new(Argon2PasswordService::new());
        let token_hasher = Arc::new(Sha256TokenHasher::new());
        let email_service = Arc::new(MockEmailService);

        let auth_service = Arc::new(AuthService::new(
            "test-auth-service",
            credential_repo.clone(),
            session_repo.clone(),
            password_service,
            token_hasher,
            event_publisher,
            jwt_manager.clone(),
            user_profile_client,
            email_service.clone(),
        ));

        let auth_state = AuthState::new(auth_service, jwt_manager, credential_repo, session_repo);

        let metrics = get_or_init_metrics();

        let shared_state = SharedState {
            db: pool.clone(),
            redis: redis_conn,
            metrics,
            email_service: email_service.clone(),
        };

        let app_state = AppState {
            auth: auth_state,
            shared: shared_state,
        };

        // 6. Build router (same as production, but with minimal layers)
        let health_check = Arc::new(HealthCheck::new(
            app_state.shared.db.clone(),
            app_state.shared.redis.clone(),
        ));
        let cors = CorsLayer::permissive();
        let trace = TraceLayer::new_for_http();

        let router = auth_service::presentation::http::routes::create_router(
            app_state,
            health_check,
            cors,
            trace,
        );

        Self {
            router,
            pool,
            _pg_container: pg_container,
            _redis_container: redis_container,
            _nats_container: nats_container,
        }
    }
}
