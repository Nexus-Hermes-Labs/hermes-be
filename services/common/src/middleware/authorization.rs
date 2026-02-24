use crate::infrastructure::security::jwt_manager::{Claims, SystemRole};
use crate::middleware::authentication::AuthError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;

// ── Role-checking helpers ──────────────────────────────────────────────────

/// Returns true when the role in `claims` is in the `allowed` list.
pub fn has_role(claims: &Claims, allowed: &[SystemRole]) -> bool {
    allowed.contains(&claims.role)
}

// ── Middleware functions (use with axum::middleware::from_fn) ─────────────

/// Middleware that requires Admin role.
/// Must be applied AFTER `auth_middleware` (which sets Claims in extensions).
///
/// Usage in route builder (note order — auth runs first as outer layer):
/// ```rust
/// router
///     .route_layer(from_fn(require_admin))                  // inner: runs after auth
///     .route_layer(from_fn_with_state(jwt, auth_middleware)) // outer: runs first
/// ```
pub async fn require_admin(req: Request, next: Next) -> Result<Response, AuthError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AuthError::Unauthorized)?;

    if claims.role.is_admin() {
        Ok(next.run(req).await)
    } else {
        Err(AuthError::InsufficientPermissions)
    }
}

/// Middleware that requires Moderator or Admin role.
pub async fn require_moderator(req: Request, next: Next) -> Result<Response, AuthError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AuthError::Unauthorized)?;

    if claims.role.is_moderator_or_above() {
        Ok(next.run(req).await)
    } else {
        Err(AuthError::InsufficientPermissions)
    }
}

// ── Extractors ────────────────────────────────────────────────────────────

/// Extractor that passes only when the authenticated user has `Admin` role.
///
/// Usage in handlers:
/// ```rust
/// async fn admin_handler(
///     _: AdminOnly,
///     AuthenticatedUser(claims): AuthenticatedUser,
/// ) -> impl IntoResponse { ... }
/// ```
pub struct AdminOnly;

#[async_trait]
impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        if claims.role.is_admin() {
            Ok(AdminOnly)
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}

/// Extractor that passes when the authenticated user has `Moderator` or `Admin` role.
pub struct ModeratorOrAbove;

#[async_trait]
impl<S> FromRequestParts<S> for ModeratorOrAbove
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        if claims.role.is_moderator_or_above() {
            Ok(ModeratorOrAbove)
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::security::jwt_manager::SystemRole;

    fn claims_with_role(role: SystemRole) -> Claims {
        use crate::infrastructure::security::jwt_manager::TokenType;
        Claims {
            iss: "test".into(),
            sub: uuid::Uuid::new_v4().to_string(),
            aud: "api".into(),
            exp: i64::MAX,
            nbf: 0,
            iat: 0,
            jti: uuid::Uuid::new_v4().to_string(),
            typ: TokenType::Access,
            email: "test@example.com".into(),
            role,
        }
    }

    #[test]
    fn test_has_role_admin_only() {
        let admin = claims_with_role(SystemRole::Admin);
        assert!(has_role(&admin, &[SystemRole::Admin]));
        assert!(!has_role(&admin, &[SystemRole::User]));
    }

    #[test]
    fn test_has_role_moderator_or_above() {
        let moderator = claims_with_role(SystemRole::Moderator);
        let admin = claims_with_role(SystemRole::Admin);
        let user = claims_with_role(SystemRole::User);

        assert!(has_role(
            &moderator,
            &[SystemRole::Moderator, SystemRole::Admin]
        ));
        assert!(has_role(
            &admin,
            &[SystemRole::Moderator, SystemRole::Admin]
        ));
        assert!(!has_role(
            &user,
            &[SystemRole::Moderator, SystemRole::Admin]
        ));
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
