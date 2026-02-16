use user_service::application::services::{
    UserPrivacyService, UserProfileService, UserRelationshipService,
};
use user_service::infrastructure::persistence::postgres::{
    PostgresUserPrivacyRepository, PostgresUserProfileRepository, PostgresUserRelationshipRepository,
};
use user_service::state::user_state::UserState;
use user_service::state::shared_state::SharedState;
use user_service::state::AppState;
use common::observability::{HealthCheck, Metrics};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// SQL migrations
const USER_ENUMS_SQL: &str =
    include_str!("../../migrations/20260121135148_create_enums.sql");
const USER_PROFILES_SQL: &str =
    include_str!("../../migrations/20260121135149_create_user_profiles.sql");
const USER_PRIVACY_SQL: &str =
    include_str!("../../migrations/20260121135150_create_user_privacy_settings.sql");
const USER_BADGES_SQL: &str =
    include_str!("../../migrations/20260121135151_create_user_badges.sql");
const USER_RELATIONSHIPS_SQL: &str =
    include_str!("../../migrations/20260121135152_create_user_relationships.sql");
const USER_INDEXES_SQL: &str =
    include_str!("../../migrations/20260121135153_create_indexes.sql");
const USER_FUNCTIONS_SQL: &str =
    include_str!("../../migrations/20260121135154_create_functions.sql");
const USER_TRIGGERS_SQL: &str =
    include_str!("../../migrations/20260121135155_create_triggers.sql");

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
    unsafe { METRICS.clone().expect("metrics initialized") }
}

/// Holds all testcontainers and the fully-built router.
///
/// Containers are dropped (and stopped) when `TestHarness` is dropped.
pub struct TestHarness {
    pub router: Router,
    // Keep containers alive for the test duration
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<GenericImage>,
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
        let pg_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            pg_host, pg_port
        );

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
        let redis_host = redis_container
            .get_host()
            .await
            .expect("get redis host");
        let redis_port = redis_container
            .get_host_port_ipv4(6379)
            .await
            .expect("get redis port");
        let redis_url = format!("redis://{}:{}", redis_host, redis_port);
        let redis_client =
            redis::Client::open(redis_url.as_str()).expect("create redis client");
        let redis_conn = redis::aio::ConnectionManager::new(redis_client)
            .await
            .expect("create redis connection manager");

        // 3. Build services
        let user_profile_repo = Arc::new(PostgresUserProfileRepository::new(pool.clone()));
        let user_privacy_repo = Arc::new(PostgresUserPrivacyRepository::new(pool.clone()));
        let relationship_repo =
            Arc::new(PostgresUserRelationshipRepository::new(pool.clone()));

        let user_profile_service = Arc::new(UserProfileService::new(user_profile_repo));
        let user_privacy_service = Arc::new(UserPrivacyService::new(user_privacy_repo));
        let relationship_service = Arc::new(UserRelationshipService::new(
            relationship_repo,
            user_privacy_service.clone(),
        ));

        let user_state = UserState::new(
            user_profile_service,
            user_privacy_service,
            relationship_service,
        );

        let metrics = get_or_init_metrics();

        let shared_state = SharedState {
            db: pool.clone(),
            redis: redis_conn.clone(),
            metrics,
        };

        let app_state = AppState {
            user: user_state,
            shared: shared_state,
        };

        // 4. Build router (same as production, but with minimal layers)
        let health_check = Arc::new(HealthCheck::new(pool, redis_conn));
        let cors = CorsLayer::permissive();
        let trace = TraceLayer::new_for_http();

        let router = user_service::presentation::http::routes::create_router(
            app_state, health_check, cors, trace,
        );

        Self {
            router,
            _pg_container: pg_container,
            _redis_container: redis_container,
        }
    }
}
