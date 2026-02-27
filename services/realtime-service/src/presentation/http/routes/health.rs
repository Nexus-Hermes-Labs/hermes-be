//! Simple health-check route — no database dependency.

use axum::{http::StatusCode, routing::get, Router};
use serde_json::json;

/// Return a lightweight 200 OK JSON response.
async fn health() -> (StatusCode, axum::Json<serde_json::Value>) {
    (StatusCode::OK, axum::Json(json!({"status": "ok"})))
}

/// Router for `/health`.
pub fn routes() -> Router {
    Router::new().route("/", get(health))
}
