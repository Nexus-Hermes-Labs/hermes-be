use axum::Router;
use chat_service::application::ports::ChatUnitOfWorkFactory;
use chat_service::application::{MessageService, ReactionService};
use chat_service::infrastructure::persistence::postgres::PgChatUnitOfWorkFactory;
use chat_service::infrastructure::persistence::{
    PostgresMessageRepository, PostgresReactionRepository,
};
use chat_service::state::{AppState, ChatState, SharedState};
use common::observability::{request_trace_layer, HealthCheck, Metrics};
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tower_http::cors::CorsLayer;

// ============================================
// SQL MIGRATIONS (inlined — chat-service uses
// the shared messages + reactions tables)
// ============================================

const ENUMS_SQL: &str = "
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TYPE message_type      AS ENUM ('text', 'system');
CREATE TYPE conversation_type AS ENUM ('dm', 'group_dm');
";

const CONVERSATIONS_SQL: &str = "
CREATE TABLE conversations (
    id         UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    type       conversation_type NOT NULL,
    created_at TIMESTAMPTZ       NOT NULL DEFAULT NOW()
);
CREATE TABLE conversation_members (
    conversation_id UUID        NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, user_id)
);
";

const MESSAGES_SQL: &str = "
CREATE TABLE messages (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id      UUID,
    conversation_id UUID         REFERENCES conversations(id) ON DELETE CASCADE,
    user_id         UUID         NOT NULL,
    content         TEXT         NOT NULL CHECK (char_length(content) BETWEEN 1 AND 2000),
    type            message_type NOT NULL DEFAULT 'text',
    reply_to_id     UUID         REFERENCES messages(id) ON DELETE SET NULL,
    is_deleted      BOOLEAN      NOT NULL DEFAULT FALSE,
    edited_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_message_target CHECK (
        (channel_id IS NOT NULL AND conversation_id IS NULL) OR
        (channel_id IS NULL     AND conversation_id IS NOT NULL)
    )
);
";

const REACTIONS_SQL: &str = "
CREATE TABLE reactions (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID        NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL,
    emoji      TEXT        NOT NULL CHECK (char_length(emoji) BETWEEN 1 AND 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (message_id, user_id, emoji)
);
";

const INDEXES_SQL: &str = "
CREATE INDEX idx_messages_channel_id   ON messages (channel_id)   WHERE channel_id IS NOT NULL;
CREATE INDEX idx_messages_conversation ON messages (conversation_id) WHERE conversation_id IS NOT NULL;
CREATE INDEX idx_reactions_message_id  ON reactions (message_id);
";

const OUTBOX_SQL: &str = "
CREATE TABLE outbox_events (
    id              UUID         PRIMARY KEY,
    aggregate_id    UUID         NOT NULL,
    aggregate_type  TEXT         NOT NULL
        CHECK (aggregate_type IN ('chat_message', 'chat_reaction')),
    event_type      TEXT         NOT NULL
        CHECK (event_type IN (
            'chat.message.created',
            'chat.message.updated',
            'chat.message.deleted',
            'chat.reaction.added',
            'chat.reaction.removed'
        )),
    payload         JSONB        NOT NULL,
    source_service  TEXT         NOT NULL,
    status          TEXT         NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'published', 'failed')),
    retry_count     INTEGER      NOT NULL DEFAULT 0
        CHECK (retry_count >= 0),
    last_error      TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    next_retry_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ
);
CREATE INDEX idx_outbox_events_publishable
    ON outbox_events (source_service, next_retry_at)
    WHERE status IN ('pending', 'failed');
";

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
pub struct TestHarness {
    pub router: Router,
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
            ("outbox", OUTBOX_SQL),
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
        let nats_url = format!("nats://{nats_host}:{nats_port}");

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
        let _nats_client = nats_client;

        let message_repo = Arc::new(PostgresMessageRepository::new(pool.clone()));
        let reaction_repo = Arc::new(PostgresReactionRepository::new(pool.clone()));
        let uow_factory: Arc<dyn ChatUnitOfWorkFactory> = Arc::new(PgChatUnitOfWorkFactory::new(
            pool.clone(),
            "chat-service-test",
        ));

        let message_service = Arc::new(MessageService::new(
            "chat-service-test",
            message_repo.clone() as Arc<dyn chat_service::domain::MessageRepository>,
            uow_factory.clone(),
        ));
        let reaction_service = Arc::new(ReactionService::new(
            "chat-service-test",
            reaction_repo as Arc<dyn chat_service::domain::ReactionRepository>,
            message_repo as Arc<dyn chat_service::domain::MessageRepository>,
            uow_factory,
        ));

        // ── 5. Assemble state ────────────────────────────────────────────────
        let metrics = get_or_init_metrics();
        let chat_state = ChatState::new(message_service, reaction_service);
        let shared_state = SharedState {
            db: pool.clone(),
            redis: redis_conn.clone(),
            metrics,
        };
        let app_state = AppState {
            chat: chat_state,
            shared: shared_state,
        };

        // ── 6. Build router ──────────────────────────────────────────────────
        let health_check = Arc::new(HealthCheck::new(pool, redis_conn));
        let cors = CorsLayer::permissive();
        let trace = request_trace_layer();

        let router = chat_service::presentation::http::routes::create_router(
            app_state,
            health_check,
            cors,
            trace,
        );

        Self {
            router,
            _pg_container: pg_container,
            _redis_container: redis_container,
            _nats_container: nats_container,
        }
    }
}
