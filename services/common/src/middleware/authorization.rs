use crate::infrastructure::security::jwt_manager::SystemRole;
use crate::middleware::authentication::AuthError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use std::str::FromStr;

// ── Role-checking helpers ──────────────────────────────────────────────────

fn role_from_request(req: &Request) -> SystemRole {
    req.headers()
        .get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| SystemRole::from_str(v).ok())
        .unwrap_or_default()
}

fn role_from_parts(parts: &Parts) -> SystemRole {
    parts
        .headers
        .get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| SystemRole::from_str(v).ok())
        .unwrap_or_default()
}

// ── Middleware functions (use with axum::middleware::from_fn) ─────────────

/// Middleware that requires Admin role.
/// Reads `X-User-Role` header injected by Traefik ForwardAuth.
pub async fn require_admin(req: Request, next: Next) -> Result<Response, AuthError> {
    if role_from_request(&req).is_admin() {
        Ok(next.run(req).await)
    } else {
        Err(AuthError::InsufficientPermissions)
    }
}

/// Middleware that requires Moderator or Admin role.
pub async fn require_moderator(req: Request, next: Next) -> Result<Response, AuthError> {
    if role_from_request(&req).is_moderator_or_above() {
        Ok(next.run(req).await)
    } else {
        Err(AuthError::InsufficientPermissions)
    }
}

// ── Extractors ────────────────────────────────────────────────────────────

/// Extractor that passes only when the caller has `Admin` role.
pub struct AdminOnly;

#[async_trait]
impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if role_from_parts(parts).is_admin() {
            Ok(AdminOnly)
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}

/// Extractor that passes when the caller has `Moderator` or `Admin` role.
pub struct ModeratorOrAbove;

#[async_trait]
impl<S> FromRequestParts<S> for ModeratorOrAbove
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if role_from_parts(parts).is_moderator_or_above() {
            Ok(ModeratorOrAbove)
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::security::jwt_manager::SystemRole;

    #[test]
    fn test_has_role_admin_only() {
        assert!(SystemRole::Admin.is_admin());
        assert!(!SystemRole::User.is_admin());
    }

    #[test]
    fn test_has_role_moderator_or_above() {
        assert!(SystemRole::Moderator.is_moderator_or_above());
        assert!(SystemRole::Admin.is_moderator_or_above());
        assert!(!SystemRole::User.is_moderator_or_above());
    }

    #[test]
    fn test_system_role_is_admin() {
        assert!(SystemRole::Admin.is_admin());
        assert!(!SystemRole::Moderator.is_admin());
        assert!(!SystemRole::User.is_admin());
    }

    #[test]
    fn test_system_role_is_moderator_or_above() {
        assert!(SystemRole::Admin.is_moderator_or_above());
        assert!(SystemRole::Moderator.is_moderator_or_above());
        assert!(!SystemRole::User.is_moderator_or_above());
    }
}
