use axum::Router;
use common::observability::{HealthCheck, Metrics};
use guild_service::application::{
    GuildInviteService, GuildMemberService, GuildRoleService, GuildService,
};
use guild_service::infrastructure::persistence::{
    PostgresGuildInviteRepository, PostgresGuildMemberRepository, PostgresGuildRepository,
    PostgresGuildRoleRepository,
};
use guild_service::state::guild_state::GuildState;
use guild_service::state::shared_state::SharedState;
use guild_service::state::AppState;
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

// ============================================
// SQL MIGRATIONS (embedded at compile-time)
// ============================================

const GUILD_ENUMS_SQL: &str =
    include_str!("../../migrations/20260210000000_create_guild_enums.sql");
const GUILDS_SQL: &str = include_str!("../../migrations/20260210000001_create_guilds.sql");
const GUILD_MEMBERS_SQL: &str =
    include_str!("../../migrations/20260210000002_create_guild_members.sql");
const GUILD_ROLES_SQL: &str =
    include_str!("../../migrations/20260210000003_create_guild_roles.sql");
const GUILD_INVITES_SQL: &str =
    include_str!("../../migrations/20260210000004_create_guild_invites.sql");
const GUILD_INDEXES_SQL: &str = include_str!("../../migrations/20260210000005_create_indexes.sql");
const GUILD_MEMBER_ROLES_SQL: &str =
    include_str!("../../migrations/20260210000006_create_guild_member_roles.sql");
const GUILD_MEMBER_ROLES_INDEXES_SQL: &str =
    include_str!("../../migrations/20260210000007_create_guild_member_roles_indexes.sql");

// ============================================
// METRICS SINGLETON (init only once per process)
// ============================================

static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();

fn get_or_init_metrics() -> Metrics {
    METRICS
        .get_or_init(|| Metrics::init().expect("init metrics"))
        .clone()
}

// ============================================
// TEST HARNESS
// ============================================

/// Holds running containers and the fully-wired Axum router.
///
/// Containers are stopped when the harness is dropped at the end of each test.
pub struct TestHarness {
    pub router: Router,
    pub pool: PgPool,
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<GenericImage>,
}

impl TestHarness {
    pub async fn new() -> Self {
        // ── 1. PostgreSQL ────────────────────────────────────────────────────
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
            ("guild enums", GUILD_ENUMS_SQL),
            ("guilds", GUILDS_SQL),
            ("guild members", GUILD_MEMBERS_SQL),
            ("guild roles", GUILD_ROLES_SQL),
            ("guild invites", GUILD_INVITES_SQL),
            ("guild indexes", GUILD_INDEXES_SQL),
            ("guild member roles", GUILD_MEMBER_ROLES_SQL),
            ("guild member roles indexes", GUILD_MEMBER_ROLES_INDEXES_SQL),
        ];
        for (name, sql) in migrations {
            sqlx::raw_sql(sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("migration '{name}': {e}"));
        }

        // ── 2. Redis ─────────────────────────────────────────────────────────
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
        let redis_url = format!("redis://{redis_host}:{redis_port}");
        let redis_client = redis::Client::open(redis_url.as_str()).expect("create redis client");
        let redis_conn = redis::aio::ConnectionManager::new(redis_client)
            .await
            .expect("create redis connection manager");

        // ── 3. Build repositories ────────────────────────────────────────────
        let guild_repo: Arc<dyn guild_service::domain::guild::GuildRepository> =
            Arc::new(PostgresGuildRepository::new(pool.clone()));
        let member_repo: Arc<dyn guild_service::domain::guild_member::GuildMemberRepository> =
            Arc::new(PostgresGuildMemberRepository::new(pool.clone()));
        let role_repo: Arc<dyn guild_service::domain::guild_role::GuildRoleRepository> =
            Arc::new(PostgresGuildRoleRepository::new(pool.clone()));
        let invite_repo: Arc<dyn guild_service::domain::guild_invite::GuildInviteRepository> =
            Arc::new(PostgresGuildInviteRepository::new(pool.clone()));

        // ── 4. Build application services ───────────────────────────────────
        let guild_service = Arc::new(GuildService::new(
            guild_repo.clone(),
            member_repo.clone(),
            role_repo.clone(),
        ));
        let member_service = Arc::new(GuildMemberService::new(
            guild_repo.clone(),
            member_repo.clone(),
        ));
        let role_service = Arc::new(GuildRoleService::new(guild_repo.clone(), role_repo.clone()));
        let invite_service = Arc::new(GuildInviteService::new(
            guild_repo,
            invite_repo,
            member_repo,
        ));

        // ── 5. Assemble state ────────────────────────────────────────────────
        let metrics = get_or_init_metrics();

        let guild_state =
            GuildState::new(guild_service, member_service, role_service, invite_service);

        let user_grpc_client =
            guild_service::infrastructure::grpc::UserGrpcClient::new("http://[::1]:50052")
                .expect("create user grpc client");

        let shared_state = SharedState {
            db: pool.clone(),
            redis: redis_conn.clone(),
            metrics,
            user_grpc_client: Arc::new(user_grpc_client),
        };
        let app_state = AppState {
            guild: guild_state,
            shared: shared_state,
        };

        // ── 7. Build router ──────────────────────────────────────────────────
        let health_check = Arc::new(HealthCheck::new(pool.clone(), redis_conn));
        let cors = CorsLayer::permissive();
        let trace = TraceLayer::new_for_http();

        let router = guild_service::presentation::http::routes::create_router(
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
        }
    }

    /// Return the UUID string used as the Traefik-injected `X-User-Id` header in tests.
    #[allow(clippy::unused_self)]
    pub fn token_for(&self, user_id: Uuid) -> String {
        user_id.to_string()
    }

    /// Directly insert a member row (bypasses service layer for fixture setup).
    ///
    /// Roles are stored in `guild_member_roles`; no `role_ids` column on `guild_members`.
    /// Also increments the guild's `member_count` to stay consistent.
    pub async fn seed_member(&self, guild_id: Uuid, user_id: Uuid) {
        sqlx::query(
            "INSERT INTO guild_members (guild_id, user_id, joined_at, updated_at) \
             VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(guild_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("seed member");

        sqlx::query(
            "UPDATE guilds SET member_count = member_count + 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(guild_id)
        .execute(&self.pool)
        .await
        .expect("increment member count after seed");
    }
}
