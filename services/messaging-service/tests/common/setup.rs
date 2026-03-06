use axum::Router;
use common::observability::{HealthCheck, Metrics};
use messaging_service::application::{ConversationService, MessageService, ReactionService};
use messaging_service::infrastructure::persistence::{
    PgMessagingUnitOfWorkFactory, PostgresConversationRepository, PostgresMessageRepository,
    PostgresReactionRepository,
};
use messaging_service::infrastructure::NatsPublisher;
use messaging_service::state::messaging_state::MessagingState;
use messaging_service::state::shared_state::SharedState;
use messaging_service::state::AppState;
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// ============================================
// SQL MIGRATIONS (embedded at compile-time)
// ============================================

const ENUMS_SQL: &str =
    include_str!("../../migrations/20260226000000_create_enums.sql");
const CONVERSATIONS_SQL: &str =
    include_str!("../../migrations/20260226000001_create_conversations.sql");
const MESSAGES_SQL: &str =
    include_str!("../../migrations/20260226000002_create_messages.sql");
const REACTIONS_SQL: &str =
    include_str!("../../migrations/20260226000003_create_reactions.sql");
const INDEXES_SQL: &str =
    include_str!("../../migrations/20260226000004_create_indexes.sql");

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
    #[allow(dead_code)]
    pub pool: PgPool,
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<GenericImage>,
    _nats_container: ContainerAsync<GenericImage>,
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

        let migrations: &[(&str, &str)] = &[
            ("enums", ENUMS_SQL),
            ("conversations", CONVERSATIONS_SQL),
            ("messages", MESSAGES_SQL),
            ("reactions", REACTIONS_SQL),
            ("indexes", INDEXES_SQL),
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

        // ── 3. NATS ──────────────────────────────────────────────────────────
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

        let mut nats_client = None;
        for attempt in 0..10 {
            match async_nats::connect(&nats_url).await {
                Ok(c) => {
                    nats_client = Some(c);
                    break;
                }
                Err(_) if attempt < 9 => {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
                Err(e) => panic!("connect to nats after retries: {e}"),
            }
        }
        let nats_client = nats_client.expect("nats client");

        // ── 4. Build services ────────────────────────────────────────────────
        let nats_publisher = Arc::new(NatsPublisher::new(nats_client.clone()));

        let message_repo = Arc::new(PostgresMessageRepository::new(pool.clone()));
        let reaction_repo = Arc::new(PostgresReactionRepository::new(pool.clone()));
        let conversation_repo = Arc::new(PostgresConversationRepository::new(pool.clone()));
        let uow_factory = Arc::new(PgMessagingUnitOfWorkFactory::new(pool.clone()));

        let message_service =
            Arc::new(MessageService::new(message_repo, nats_publisher.clone()));
        let reaction_service =
            Arc::new(ReactionService::new(reaction_repo, nats_publisher.clone()));
        let conversation_service = Arc::new(ConversationService::new(
            conversation_repo,
            uow_factory,
            nats_publisher,
        ));

        // ── 5. Assemble state ────────────────────────────────────────────────
        let metrics = get_or_init_metrics();
        let messaging_state =
            MessagingState::new(message_service, reaction_service, conversation_service);
        let shared_state = SharedState {
            db: pool.clone(),
            redis: redis_conn.clone(),
            metrics,
            nats: nats_client,
        };
        let app_state = AppState {
            messaging: messaging_state,
            shared: shared_state,
        };

        // ── 6. Build router ──────────────────────────────────────────────────
        let health_check = Arc::new(HealthCheck::new(pool.clone(), redis_conn));
        let cors = CorsLayer::permissive();
        let trace = TraceLayer::new_for_http();

        let router = messaging_service::presentation::http::routes::create_router(
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
