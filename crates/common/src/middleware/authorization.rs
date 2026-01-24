use axum::http::StatusCode;
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::jwt::Claims;
use crate::middleware::authentication::{AuthError, AuthenticatedUser};

const REQUIRED_ADMIN: &str = "admin";

/// Middleware to require admin role
pub async fn require_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if claims.role != REQUIRED_ADMIN {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Generic role checker - can be used for different roles
pub async fn require_role(
    authenticated_user: AuthenticatedUser,
    required_role: &str,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let role = authenticated_user
        .0
        .role;

    if role != required_role {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

/// Check if a user has one of the allowed roles
pub fn has_role(user_role: &str, allowed_roles: &[&str]) -> bool {
    allowed_roles.contains(&user_role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_role() {
        assert!(has_role("admin", &["admin"]));
        assert!(has_role("customer", &["customer"]));
        assert!(has_role("admin", &["admin", "customer"]));
        assert!(!has_role("customer", &["admin"]));
        assert!(!has_role("invalid", &["admin"]));
    }
}
