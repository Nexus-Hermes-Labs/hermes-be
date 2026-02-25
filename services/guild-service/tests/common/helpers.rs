use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Send a JSON request to the router and return status + parsed body.
pub async fn make_json_request(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let body_str = body
        .map(|v| serde_json::to_string(&v).expect("serialize body"))
        .unwrap_or_default();

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body_str))
        .expect("build request");

    let response = app.oneshot(request).await.expect("send request");
    let status = response.status();

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();

    let value: serde_json::Value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                let body_str = String::from_utf8_lossy(&bytes);
                panic!("Failed to parse response as JSON. Error: {e}, Body: {body_str}");
            }
        }
    };

    (status, value)
}

/// Send an authenticated JSON request with Traefik `ForwardAuth` identity headers.
///
/// `user_id` must be a valid UUID string — Traefik injects these headers after
/// a successful `/internal/verify` check; in tests we inject them directly.
pub async fn make_authenticated_request(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
    user_id: &str,
) -> (StatusCode, serde_json::Value) {
    let body_str = body
        .map(|v| serde_json::to_string(&v).expect("serialize body"))
        .unwrap_or_default();

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-user-id", user_id)
        .header("x-user-role", "user")
        .header("x-user-email", format!("{user_id}@test.com"))
        .body(axum::body::Body::from(body_str))
        .expect("build request");

    let response = app.oneshot(request).await.expect("send request");
    let status = response.status();

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();

    let value: serde_json::Value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                let body_str = String::from_utf8_lossy(&bytes);
                panic!("Failed to parse response as JSON. Error: {e}, Body: {body_str}");
            }
        }
    };

    (status, value)
}
