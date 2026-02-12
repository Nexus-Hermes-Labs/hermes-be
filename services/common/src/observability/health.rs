use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: ComponentHealth,
    redis: ComponentHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Clone)]
pub struct HealthCheck {
    pool: PgPool,
    redis: redis::aio::ConnectionManager,
}

impl HealthCheck {
    pub fn new(pool: PgPool, redis: redis::aio::ConnectionManager) -> Self {
        Self { pool, redis }
    }

    pub async fn check(&self) -> HealthResponse {
        let db_health = self.check_database().await;
        let redis_health = self.check_redis().await;

        let overall_status = if db_health.status == "ok" && redis_health.status == "ok" {
            "ok"
        } else {
            "degraded"
        };

        HealthResponse {
            status: overall_status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: HealthChecks {
                database: db_health,
                redis: redis_health,
            },
        }
    }

    async fn check_database(&self) -> ComponentHealth {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => ComponentHealth {
                status: "ok".to_string(),
                message: None,
            },
            Err(e) => ComponentHealth {
                status: "error".to_string(),
                message: Some(e.to_string()),
            },
        }
    }

    async fn check_redis(&self) -> ComponentHealth {
        let mut conn = self.redis.clone();
        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
            Ok(_) => ComponentHealth {
                status: "ok".to_string(),
                message: None,
            },
            Err(e) => ComponentHealth {
                status: "error".to_string(),
                message: Some(e.to_string()),
            },
        }
    }
}

pub async fn health_handler(State(health_check): State<Arc<HealthCheck>>) -> impl IntoResponse {
    let response = health_check.check().await;
    let status = if response.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

pub async fn liveness_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn readiness_handler(State(health_check): State<Arc<HealthCheck>>) -> impl IntoResponse {
    let response = health_check.check().await;
    if response.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
