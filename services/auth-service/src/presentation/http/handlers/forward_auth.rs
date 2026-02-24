use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
};

/// GET /internal/verify
///
/// ForwardAuth endpoint called by Traefik on every protected request.
/// Validates the JWT access token and forwards user identity headers on success.
pub async fn verify_handler<B>(
    State(state): State<AppState>,
    request: Request<B>,
) -> Response {
    let token = match extract_bearer_token(request.headers()) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    match state.auth.jwt_manager.verify_access_token(token) {
        Ok(claims) => {
            let user_id = match claims.user_id() {
                Ok(id) => id.to_string(),
                Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
            };

            let mut headers = HeaderMap::new();

            // These headers are forwarded by Traefik to the upstream service
            if let Ok(val) = user_id.parse() {
                headers.insert("x-user-id", val);
            }
            if let Ok(val) = claims.role.as_str().parse() {
                headers.insert("x-user-role", val);
            }
            if let Ok(val) = claims.email.parse() {
                headers.insert("x-user-email", val);
            }

            (StatusCode::OK, headers).into_response()
        }
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}
