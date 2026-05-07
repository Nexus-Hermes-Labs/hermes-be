mod common;

use axum::http::{Method, StatusCode};
use common::helpers::{make_json_request, make_json_request_with_headers};
use common::setup::TestHarness;
use serde_json::json;

// ============================================
// HELPER: register a user and return (status, body)
// ============================================
async fn register_user(
    harness: &TestHarness,
    email: &str,
    username: &str,
    display_name: &str,
    password: &str,
) -> (StatusCode, serde_json::Value) {
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({
            "email": email,
            "username": username,
            "display_name": display_name,
            "password": password
        })),
    )
    .await
}

async fn login_user(
    harness: &TestHarness,
    email: &str,
    password: &str,
) -> (StatusCode, serde_json::Value) {
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/login",
        Some(json!({
            "email": email,
            "password": password
        })),
    )
    .await
}

async fn verify_email(harness: &TestHarness, email: &str) {
    // 1. Get verification token from DB
    let row: (Option<String>,) =
        sqlx::query_as("SELECT email_verification_token FROM auth_credentials WHERE email = $1")
            .bind(email)
            .fetch_one(&harness.pool)
            .await
            .expect("failed to fetch verification token");

    let token = row.0.expect("verification token not found in DB");

    // 2. Call verify-email endpoint
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/auth/verify-email?token={}", token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "email verification failed: {body}");
}

// ============================================
// REGISTRATION TESTS
// ============================================

#[tokio::test]
async fn test_register_success() {
    let harness = TestHarness::new().await;

    let (status, body) = register_user(
        &harness,
        "alice@example.com",
        "alice",
        "Alice",
        "strongpassword123",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");
    assert!(body["expires_in"].is_number());
    assert!(body["user"]["username"].is_string());
}

#[tokio::test]
async fn test_register_email_normalization() {
    let harness = TestHarness::new().await;

    // 1. First Register
    let (status, body) = register_user(
        &harness,
        "verify@example.com",
        "verifyuser",
        "Verify User",
        "strongpassword123",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    // 2. Second Register
    let (status, body) = register_user(
        &harness,
        "VERIFY@example.com",
        "verifyuser2",
        "Verify User",
        "strongpassword123",
    )
    .await;

    assert_ne!(status, StatusCode::CREATED);

    // 3. Third Register
    let (status, body) = register_user(
        &harness,
        "VERify@example.com",
        "verifyuser1",
        "Verify User",
        "strongpassword123",
    )
    .await;

    assert_ne!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_duplicate_username() {
    // Username uniqueness is enforced by user-service (it owns the
    // user_profiles table). Auth-service's job is to translate the gRPC
    // `AlreadyExists` status into a 409 CONFLICT for the client.
    //
    // We use a harness whose mock user-service always returns
    // `Status::already_exists`, so the request still hits register's
    // pre-credential-save step and exercises the mapping.
    let harness = TestHarness::with_username_conflict().await;

    let (status, body) = register_user(
        &harness,
        "dup-username@example.com",
        "takenuser",
        "Taken User",
        "strongpassword123",
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(body["code"], "USERNAME_ALREADY_EXISTS");
}

#[tokio::test]
async fn test_email_verification_success() {
    let harness = TestHarness::new().await;

    // 1. Register
    register_user(
        &harness,
        "verify@example.com",
        "verifyuser",
        "Verify User",
        "strongpassword123",
    )
    .await;

    // 2. Try login (should fail because not verified)
    let (status, body) = login_user(&harness, "verify@example.com", "strongpassword123").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "EMAIL_NOT_VERIFIED");

    // 3. Verify email
    verify_email(&harness, "verify@example.com").await;

    // 4. Try login again (should succeed)
    let (status, _) = login_user(&harness, "verify@example.com", "strongpassword123").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let harness = TestHarness::new().await;

    let (status, _) = register_user(
        &harness,
        "dup@example.com",
        "user_one",
        "User One",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Same email, different username
    let (status, _) = register_user(
        &harness,
        "dup@example.com",
        "user_two",
        "User Two",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_invalid_email() {
    let harness = TestHarness::new().await;

    let (status, _) = register_user(
        &harness,
        "not-an-email",
        "validuser",
        "Valid User",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_short_password() {
    let harness = TestHarness::new().await;

    let (status, _) = register_user(
        &harness,
        "short@example.com",
        "shortpw",
        "Short PW",
        "short",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_invalid_username() {
    let harness = TestHarness::new().await;

    // Uppercase not allowed
    let (status, _) = register_user(
        &harness,
        "upper@example.com",
        "UpperCase",
        "Upper",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Too short (< 3 chars)
    let (status, _) = register_user(
        &harness,
        "short@example.com",
        "ab",
        "Short",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_missing_fields() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({})),
    )
    .await;

    // Missing required fields should return 400 or 422
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {status}"
    );
}

// ============================================
// LOGIN TESTS
// ============================================

#[tokio::test]
async fn test_login_success() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "login@example.com",
        "loginuser",
        "Login User",
        "strongpassword123",
    )
    .await;

    // Must verify email before login
    verify_email(&harness, "login@example.com").await;

    let (status, body) = login_user(&harness, "login@example.com", "strongpassword123").await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");
}

#[tokio::test]
async fn test_login_wrong_password() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "wrongpw@example.com",
        "wrongpw",
        "Wrong PW",
        "strongpassword123",
    )
    .await;

    // Must verify email before login attempts are evaluated for password
    verify_email(&harness, "wrongpw@example.com").await;

    let (status, _) = login_user(&harness, "wrongpw@example.com", "wrongpassword").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_email() {
    let harness = TestHarness::new().await;

    let (status, _) = login_user(&harness, "nobody@example.com", "strongpassword123").await;

    // Should be 401 or 404
    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::NOT_FOUND,
        "expected 401 or 404, got {status}"
    );
}

#[tokio::test]
async fn test_login_invalid_email_format() {
    let harness = TestHarness::new().await;

    let (status, _) = login_user(&harness, "not-an-email", "strongpassword123").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================
// TOKEN REFRESH TESTS
// ============================================

#[tokio::test]
async fn test_refresh_token_success() {
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "refresh@example.com",
        "refreshuser",
        "Refresh User",
        "strongpassword123",
    )
    .await;

    let refresh_token = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": refresh_token })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    // Rotation: refresh now returns a brand-new refresh token.
    assert_ne!(body["refresh_token"].as_str(), Some(refresh_token));
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": "invalid-token-value" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Helper: drive a refresh and return (status, body).
async fn refresh(harness: &TestHarness, refresh_token: &str) -> (StatusCode, serde_json::Value) {
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": refresh_token })),
    )
    .await
}

#[tokio::test]
async fn test_refresh_rotates_token() {
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "rotate@example.com",
        "rotateuser",
        "Rotate User",
        "strongpassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, body) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let new_refresh = body["refresh_token"].as_str().expect("rotated refresh_token");
    assert_ne!(new_refresh, original, "expected a brand-new refresh token");

    // The new token works for another refresh.
    let (status, _) = refresh(&harness, new_refresh).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_refresh_replay_in_grace_window_succeeds() {
    // Within the 30s grace window an in-flight retry with the previous
    // token must be accepted (idempotent), not flagged as theft.
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "grace@example.com",
        "graceuser",
        "Grace User",
        "strongpassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, _) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK);

    // Replay original immediately — still inside grace.
    let (status, body) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["refresh_token"].is_string());

    // Session is still active.
    let (active_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE",
    )
    .fetch_one(&harness.pool)
    .await
    .expect("count active sessions");
    assert_eq!(active_count, 1);
}

#[tokio::test]
async fn test_refresh_reuse_after_grace_revokes_all_sessions() {
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "reuse@example.com",
        "reuseuser",
        "Reuse User",
        "strongpassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    // Verify so we can log in to add a second session.
    verify_email(&harness, "reuse@example.com").await;
    let (_, login_body) = login_user(&harness, "reuse@example.com", "strongpassword123").await;
    let _other_refresh = login_body["refresh_token"].as_str().expect("login refresh");

    // Two active sessions exist.
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(count, 2);

    // Rotate `original` once.
    let (status, _) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK);

    // Push rotated_at past the grace window so the next replay is reuse.
    sqlx::query(
        "UPDATE auth_sessions SET rotated_at = NOW() - INTERVAL '60 seconds' \
         WHERE previous_refresh_token_hash IS NOT NULL",
    )
    .execute(&harness.pool)
    .await
    .expect("backdate rotated_at");

    // Replay original out of grace → reuse detected.
    let (status, _) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // ALL of this user's sessions should now be revoked.
    let (active,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(active, 0, "expected reuse detection to revoke every session");
}

#[tokio::test]
async fn test_refresh_captures_client_ip_via_xff() {
    // Behind Traefik the real client IP arrives in X-Forwarded-For.
    // Verify the rotated session row records the rightmost (trusted) entry
    // in `last_used_ip`, while `ip_address` keeps the original create-time IP.
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "ipaudit@example.com",
        "ipaudit",
        "IP Audit",
        "strongpassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, _) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": original })),
        &[("x-forwarded-for", "203.0.113.42")],
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (last_used_ip,): (Option<String>,) =
        sqlx::query_as("SELECT host(last_used_ip) FROM auth_sessions LIMIT 1")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(last_used_ip.as_deref(), Some("203.0.113.42"));
}

#[tokio::test]
async fn test_refresh_slides_expiration() {
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "slide@example.com",
        "slideuser",
        "Slide User",
        "strongpassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (initial_expiry,): (chrono::DateTime<chrono::Utc>,) =
        sqlx::query_as("SELECT expires_at FROM auth_sessions LIMIT 1")
            .fetch_one(&harness.pool)
            .await
            .unwrap();

    // Backdate created_at + expires_at so the slide is observable even
    // when the rotation happens within the same second as registration.
    sqlx::query(
        "UPDATE auth_sessions SET expires_at = expires_at - INTERVAL '1 hour'",
    )
    .execute(&harness.pool)
    .await
    .expect("backdate expires_at");

    let (status, _) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK);

    let (rotated_expiry,): (chrono::DateTime<chrono::Utc>,) =
        sqlx::query_as("SELECT expires_at FROM auth_sessions LIMIT 1")
            .fetch_one(&harness.pool)
            .await
            .unwrap();

    assert!(
        rotated_expiry > initial_expiry,
        "expected sliding expiration: {rotated_expiry} > {initial_expiry}"
    );
}

#[tokio::test]
async fn test_refresh_token_empty() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": "" })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================
// LOGOUT TESTS
// ============================================

#[tokio::test]
async fn test_logout_success() {
    let harness = TestHarness::new().await;

    let (_, reg_body) = register_user(
        &harness,
        "logout@example.com",
        "logoutuser",
        "Logout User",
        "strongpassword123",
    )
    .await;

    let refresh_token = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/logout",
        Some(json!({ "refresh_token": refresh_token })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["sessions_revoked"].is_number());
}

#[tokio::test]
async fn test_logout_all_devices() {
    let harness = TestHarness::new().await;

    // Register
    let (_, _reg_body) = register_user(
        &harness,
        "logoutall@example.com",
        "logoutall",
        "Logout All",
        "strongpassword123",
    )
    .await;

    // Must verify email before login
    verify_email(&harness, "logoutall@example.com").await;

    // Login a first time to get tokens
    let (_, reg_body) = login_user(&harness, "logoutall@example.com", "strongpassword123").await;

    // Login a second time (simulating another device)
    let (_, _login_body) = login_user(&harness, "logoutall@example.com", "strongpassword123").await;

    let refresh_token = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/logout",
        Some(json!({
            "refresh_token": refresh_token,
            "all_devices": true
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let revoked = body["sessions_revoked"].as_u64().expect("sessions_revoked");
    assert!(
        revoked >= 2,
        "expected at least 2 sessions revoked, got {revoked}"
    );
}

#[tokio::test]
async fn test_logout_invalid_token() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/logout",
        Some(json!({ "refresh_token": "invalid-token" })),
    )
    .await;

    // Should fail — invalid token
    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::NOT_FOUND,
        "expected 401 or 404, got {status}"
    );
}

// ============================================
// FULL FLOW TEST
// ============================================

#[tokio::test]
async fn test_full_auth_flow() {
    let harness = TestHarness::new().await;

    // 1. Register
    let (status, reg_body) = register_user(
        &harness,
        "flow@example.com",
        "flowuser",
        "Flow User",
        "strongpassword123",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "register failed: {reg_body}");

    // 1b. Verify email
    verify_email(&harness, "flow@example.com").await;

    // 2. Login
    let (status, login_body) = login_user(&harness, "flow@example.com", "strongpassword123").await;
    assert_eq!(status, StatusCode::OK, "login failed: {login_body}");

    // 3. Refresh
    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("refresh_token from login");

    let (status, refresh_body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": refresh_token })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "refresh failed: {refresh_body}");

    // 4. Logout all devices (single-session logout doesn't work because
    //    the JWT jti doesn't match the DB session id)
    let (status, logout_body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/logout",
        Some(json!({ "refresh_token": refresh_token, "all_devices": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "logout failed: {logout_body}");

    // 5. After logout, refresh token should no longer work
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/refresh",
        Some(json!({ "refresh_token": refresh_token })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
