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
        "StrongPassword123",
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
    let (status, _body) = register_user(
        &harness,
        "verify@example.com",
        "verifyuser",
        "Verify User",
        "StrongPassword123",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {_body}");

    // 2. Second Register
    let (status, _body) = register_user(
        &harness,
        "VERIFY@example.com",
        "verifyuser2",
        "Verify User",
        "StrongPassword123",
    )
    .await;

    assert_ne!(status, StatusCode::CREATED);

    // 3. Third Register
    let (status, _body) = register_user(
        &harness,
        "VERify@example.com",
        "verifyuser1",
        "Verify User",
        "StrongPassword123",
    )
    .await;

    assert_ne!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_register_writes_user_created_outbox_event() {
    let harness = TestHarness::new().await;

    let (status, body) = register_user(
        &harness,
        "outbox@example.com",
        "outboxuser",
        "Outbox User",
        "StrongPassword123",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    let (event_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM outbox_events WHERE event_type = 'user.created'")
            .fetch_one(&harness.pool)
            .await
            .expect("count outbox events");

    assert_eq!(event_count, 1);
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
        "StrongPassword123",
    )
    .await;

    // 2. Try login (should fail because not verified)
    let (status, body) = login_user(&harness, "verify@example.com", "StrongPassword123").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "EMAIL_NOT_VERIFIED");

    // 3. Verify email
    verify_email(&harness, "verify@example.com").await;

    // 4. Try login again (should succeed)
    let (status, _) = login_user(&harness, "verify@example.com", "StrongPassword123").await;
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
        "StrongPassword123",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Same email, different username
    let (status, _) = register_user(
        &harness,
        "dup@example.com",
        "user_two",
        "User Two",
        "StrongPassword123",
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
        "StrongPassword123",
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
        "StrongPassword123",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Too short (< 3 chars)
    let (status, _) = register_user(
        &harness,
        "short@example.com",
        "ab",
        "Short",
        "StrongPassword123",
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
        "StrongPassword123",
    )
    .await;

    // Must verify email before login
    verify_email(&harness, "login@example.com").await;

    let (status, body) = login_user(&harness, "login@example.com", "StrongPassword123").await;

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
        "StrongPassword123",
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

    let (status, _) = login_user(&harness, "nobody@example.com", "StrongPassword123").await;

    // Should be 401 or 404
    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::NOT_FOUND,
        "expected 401 or 404, got {status}"
    );
}

#[tokio::test]
async fn test_login_invalid_email_format() {
    let harness = TestHarness::new().await;

    let (status, _) = login_user(&harness, "not-an-email", "StrongPassword123").await;
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
        "StrongPassword123",
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
        "StrongPassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    let (status, body) = refresh(&harness, original).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let new_refresh = body["refresh_token"]
        .as_str()
        .expect("rotated refresh_token");
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
        "StrongPassword123",
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
    let (active_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE")
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
        "StrongPassword123",
    )
    .await;
    let original = reg_body["refresh_token"].as_str().expect("refresh_token");

    // Verify so we can log in to add a second session.
    verify_email(&harness, "reuse@example.com").await;
    let (_, login_body) = login_user(&harness, "reuse@example.com", "StrongPassword123").await;
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
    assert_eq!(
        active, 0,
        "expected reuse detection to revoke every session"
    );
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
        "StrongPassword123",
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
        "StrongPassword123",
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
    sqlx::query("UPDATE auth_sessions SET expires_at = expires_at - INTERVAL '1 hour'")
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
        "StrongPassword123",
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
        "StrongPassword123",
    )
    .await;

    // Must verify email before login
    verify_email(&harness, "logoutall@example.com").await;

    // Login a first time to get tokens
    let (_, reg_body) = login_user(&harness, "logoutall@example.com", "StrongPassword123").await;

    // Login a second time (simulating another device)
    let (_, _login_body) = login_user(&harness, "logoutall@example.com", "StrongPassword123").await;

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
        "StrongPassword123",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "register failed: {reg_body}");

    // 1b. Verify email
    verify_email(&harness, "flow@example.com").await;

    // 2. Login
    let (status, login_body) = login_user(&harness, "flow@example.com", "StrongPassword123").await;
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

// ============================================
// PASSWORD POLICY TESTS
// ============================================

#[tokio::test]
async fn test_register_password_policy_no_uppercase() {
    let harness = TestHarness::new().await;

    let (status, body) = register_user(
        &harness,
        "nouppercase@example.com",
        "nouppercase",
        "No Upper",
        "alllowercase123",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_POLICY_VIOLATION");
}

#[tokio::test]
async fn test_register_password_policy_no_digit() {
    let harness = TestHarness::new().await;

    let (status, body) = register_user(
        &harness,
        "nodigit@example.com",
        "nodigit",
        "No Digit",
        "AllLettersNoDigit",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_POLICY_VIOLATION");
}

#[tokio::test]
async fn test_register_password_policy_no_lowercase() {
    let harness = TestHarness::new().await;

    let (status, body) = register_user(
        &harness,
        "nolower@example.com",
        "nolower",
        "No Lower",
        "ALLUPPERCASE123",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_POLICY_VIOLATION");
}

// ============================================
// FORGOT PASSWORD TESTS
// ============================================

#[tokio::test]
async fn test_forgot_password_existing_user() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "forgot@example.com",
        "forgotuser",
        "Forgot User",
        "StrongPassword123",
    )
    .await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "forgot@example.com" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["message"].is_string());

    // Verify token was written to DB
    let row: (Option<String>,) =
        sqlx::query_as("SELECT password_reset_token FROM auth_credentials WHERE email = $1")
            .bind("forgot@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("fetch reset token");

    assert!(row.0.is_some(), "password_reset_token should be set in DB");
}

#[tokio::test]
async fn test_forgot_password_nonexistent_user_returns_ok() {
    let harness = TestHarness::new().await;

    // Should still return 200 to prevent email enumeration
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "nobody@example.com" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn test_forgot_password_invalid_email() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "not-an-email" })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================
// RESET PASSWORD TESTS
// ============================================

#[tokio::test]
async fn test_reset_password_success() {
    let harness = TestHarness::new().await;

    // 1. Register
    register_user(
        &harness,
        "reset@example.com",
        "resetuser",
        "Reset User",
        "StrongPassword123",
    )
    .await;

    // 2. Forgot password
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "reset@example.com" })),
    )
    .await;

    // 3. Get token from DB
    let row: (Option<String>,) =
        sqlx::query_as("SELECT password_reset_token FROM auth_credentials WHERE email = $1")
            .bind("reset@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("fetch reset token");
    let token = row.0.expect("reset token should exist");

    // 4. Reset password
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/reset-password",
        Some(json!({
            "token": token,
            "new_password": "NewStrongPassword456"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");

    // 5. Verify email and login with new password
    verify_email(&harness, "reset@example.com").await;

    let (status, _) = login_user(&harness, "reset@example.com", "NewStrongPassword456").await;
    assert_eq!(status, StatusCode::OK);

    // 6. Old password should not work
    let (status, _) = login_user(&harness, "reset@example.com", "StrongPassword123").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    let harness = TestHarness::new().await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/reset-password",
        Some(json!({
            "token": "invalid-token-12345",
            "new_password": "NewStrongPassword456"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body: {body}");
}

#[tokio::test]
async fn test_reset_password_policy_violation() {
    let harness = TestHarness::new().await;

    // Register + forgot password
    register_user(
        &harness,
        "resetpolicy@example.com",
        "resetpolicy",
        "Reset Policy",
        "StrongPassword123",
    )
    .await;

    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "resetpolicy@example.com" })),
    )
    .await;

    let row: (Option<String>,) =
        sqlx::query_as("SELECT password_reset_token FROM auth_credentials WHERE email = $1")
            .bind("resetpolicy@example.com")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    let token = row.0.unwrap();

    // Try to reset with weak password (no uppercase)
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/reset-password",
        Some(json!({
            "token": token,
            "new_password": "weakpassword123"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_POLICY_VIOLATION");
}

#[tokio::test]
async fn test_reset_password_revokes_sessions() {
    let harness = TestHarness::new().await;

    // Register + verify + login to create a session
    register_user(
        &harness,
        "revokesess@example.com",
        "revokesess",
        "Revoke Sess",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "revokesess@example.com").await;
    login_user(&harness, "revokesess@example.com", "StrongPassword123").await;

    // Count active sessions before reset
    let (before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert!(before >= 2, "expected at least 2 active sessions");

    // Forgot + reset
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "revokesess@example.com" })),
    )
    .await;

    let row: (Option<String>,) =
        sqlx::query_as("SELECT password_reset_token FROM auth_credentials WHERE email = $1")
            .bind("revokesess@example.com")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    let token = row.0.unwrap();

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/reset-password",
        Some(json!({
            "token": token,
            "new_password": "BrandNewPassword789"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // All sessions should be revoked
    let (after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_sessions WHERE is_revoked = FALSE")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(after, 0, "all sessions should be revoked after password reset");
}

// ============================================
// CHANGE PASSWORD TESTS
// ============================================

#[tokio::test]
async fn test_change_password_success() {
    let harness = TestHarness::new().await;

    // Register + verify + login to get access token
    register_user(
        &harness,
        "changepw@example.com",
        "changepwuser",
        "Change PW",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "changepw@example.com").await;

    let (_, login_body) =
        login_user(&harness, "changepw@example.com", "StrongPassword123").await;
    let access_token = login_body["access_token"].as_str().expect("access_token");

    // Change password
    let (status, body) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "StrongPassword123",
            "new_password": "NewChangedPassword456"
        })),
        &[("authorization", &format!("Bearer {}", access_token))],
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");

    // Login with new password
    let (status, _) = login_user(&harness, "changepw@example.com", "NewChangedPassword456").await;
    assert_eq!(status, StatusCode::OK);

    // Old password should not work
    let (status, _) = login_user(&harness, "changepw@example.com", "StrongPassword123").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "wrongcurr@example.com",
        "wrongcurr",
        "Wrong Curr",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "wrongcurr@example.com").await;

    let (_, login_body) =
        login_user(&harness, "wrongcurr@example.com", "StrongPassword123").await;
    let access_token = login_body["access_token"].as_str().expect("access_token");

    let (status, body) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "WrongPassword999",
            "new_password": "NewStrongPassword456"
        })),
        &[("authorization", &format!("Bearer {}", access_token))],
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body: {body}");
}

#[tokio::test]
async fn test_change_password_no_auth_header() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "StrongPassword123",
            "new_password": "NewStrongPassword456"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_change_password_reuse_current() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "reusecurr@example.com",
        "reusecurr",
        "Reuse Curr",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "reusecurr@example.com").await;

    let (_, login_body) =
        login_user(&harness, "reusecurr@example.com", "StrongPassword123").await;
    let access_token = login_body["access_token"].as_str().expect("access_token");

    // Try to change to the same password
    let (status, body) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "StrongPassword123",
            "new_password": "StrongPassword123"
        })),
        &[("authorization", &format!("Bearer {}", access_token))],
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_RECENTLY_USED");
}

// ============================================
// PASSWORD HISTORY TESTS
// ============================================

#[tokio::test]
async fn test_password_history_prevents_reuse() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "history@example.com",
        "historyuser",
        "History User",
        "OriginalPassword1",
    )
    .await;
    verify_email(&harness, "history@example.com").await;

    // Change password once
    let (_, login_body) =
        login_user(&harness, "history@example.com", "OriginalPassword1").await;
    let access_token = login_body["access_token"].as_str().expect("access_token");

    let (status, _) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "OriginalPassword1",
            "new_password": "SecondPassword2"
        })),
        &[("authorization", &format!("Bearer {}", access_token))],
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Login with new password, get new token
    let (_, login_body) =
        login_user(&harness, "history@example.com", "SecondPassword2").await;
    let access_token = login_body["access_token"].as_str().expect("access_token");

    // Try to reuse the first password — should be rejected
    let (status, body) = make_json_request_with_headers(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/change-password",
        Some(json!({
            "current_password": "SecondPassword2",
            "new_password": "OriginalPassword1"
        })),
        &[("authorization", &format!("Bearer {}", access_token))],
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(body["code"], "PASSWORD_RECENTLY_USED");
}

// ============================================
// RATE LIMITING TESTS
// ============================================

#[tokio::test]
async fn test_forgot_password_rate_limit() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "ratelimit@example.com",
        "ratelimituser",
        "Rate Limit",
        "StrongPassword123",
    )
    .await;

    // Forgot password allows 3 requests per 10 minutes
    for i in 0..3 {
        let (status, _) = make_json_request(
            harness.router.clone(),
            Method::POST,
            "/api/v1/auth/forgot-password",
            Some(json!({ "email": "ratelimit@example.com" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "request {} should succeed", i + 1);
    }

    // 4th request should be rate limited
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/forgot-password",
        Some(json!({ "email": "ratelimit@example.com" })),
    )
    .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "body: {body}");
    assert_eq!(body["code"], "RATE_LIMITED");
}

#[tokio::test]
async fn test_login_rate_limit() {
    let harness = TestHarness::new().await;

    register_user(
        &harness,
        "loginrl@example.com",
        "loginrluser",
        "Login RL",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "loginrl@example.com").await;

    // Login allows 10 requests per 5 minutes
    for i in 0..10 {
        let (status, _) = login_user(&harness, "loginrl@example.com", "WrongPassword999").await;
        // Each might be 401 (wrong password) or 403 (account locked after 5 fails)
        assert!(
            status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN,
            "request {} expected 401 or 403, got {}",
            i + 1,
            status
        );
    }

    // 11th request should be rate limited
    let (status, body) = login_user(&harness, "loginrl@example.com", "WrongPassword999").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "body: {body}");
    assert_eq!(body["code"], "RATE_LIMITED");
}

// ============================================
// OAUTH (GOOGLE) — Architecture A end-to-end
//
// The flow is exercised through the real router, repos, Unit of Work and Redis
// state store; only the Google provider is faked (see common::fake_google).
// ============================================

use auth_service::application::ports::oauth_provider::OAuthUserInfo;

/// Pull the CSRF `state` out of the authorize URL the fake echoes it into.
fn extract_state(authorize_url: &str) -> String {
    authorize_url
        .split("state=")
        .nth(1)
        .expect("authorize_url contains state")
        .to_string()
}

/// Run the full Google dance: start authorization (to seed Redis state), then
/// POST the brokered `{code, state}` to the callback.
async fn google_login(harness: &TestHarness) -> (StatusCode, serde_json::Value) {
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/auth/oauth/google",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "authorize body: {body}");

    let authorize_url = body["authorize_url"]
        .as_str()
        .expect("authorize_url present");
    let state = extract_state(authorize_url);

    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/auth/oauth/google/callback",
        Some(json!({ "code": "fake-auth-code", "state": state })),
    )
    .await
}

#[tokio::test]
async fn test_oauth_google_new_user_creates_verified_credential() {
    let harness = TestHarness::new().await;
    harness.google_client.set_user_info(OAuthUserInfo {
        provider_user_id: "google-sub-new".to_string(),
        email: "newoauth@example.com".to_string(),
        email_verified: true,
        display_name: Some("New OAuth User".to_string()),
    });

    let (status, body) = google_login(&harness).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["access_token"].as_str().is_some(), "body: {body}");
    assert!(body["refresh_token"].as_str().is_some(), "body: {body}");

    // A verified credential was created.
    let (verified,): (bool,) =
        sqlx::query_as("SELECT email_verified FROM auth_credentials WHERE email = $1")
            .bind("newoauth@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("credential exists");
    assert!(verified, "OAuth credential should be email-verified");

    // The oauth link was recorded.
    let (link_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM oauth_accounts WHERE provider = 'google' AND provider_user_id = $1",
    )
    .bind("google-sub-new")
    .fetch_one(&harness.pool)
    .await
    .expect("count oauth accounts");
    assert_eq!(link_count, 1);

    // A user.created event was queued for user-service.
    let (event_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM outbox_events WHERE event_type = 'user.created'")
            .fetch_one(&harness.pool)
            .await
            .expect("count outbox events");
    assert_eq!(event_count, 1);
}

#[tokio::test]
async fn test_oauth_google_links_existing_password_account() {
    let harness = TestHarness::new().await;

    // Existing verified password user.
    register_user(
        &harness,
        "linkme@example.com",
        "linkuser",
        "Link User",
        "StrongPassword123",
    )
    .await;
    verify_email(&harness, "linkme@example.com").await;

    let (credential_id,): (uuid::Uuid,) =
        sqlx::query_as("SELECT id FROM auth_credentials WHERE email = $1")
            .bind("linkme@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("existing credential");

    // Google returns the same (verified) email under a fresh subject.
    harness.google_client.set_user_info(OAuthUserInfo {
        provider_user_id: "google-sub-link".to_string(),
        email: "linkme@example.com".to_string(),
        email_verified: true,
        display_name: Some("Link User".to_string()),
    });

    let (status, body) = google_login(&harness).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    // No second credential was created.
    let (credential_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_credentials WHERE email = $1")
            .bind("linkme@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("count credentials");
    assert_eq!(credential_count, 1);

    // The link points at the pre-existing credential.
    let (linked_credential_id,): (uuid::Uuid,) = sqlx::query_as(
        "SELECT credential_id FROM oauth_accounts WHERE provider = 'google' AND provider_user_id = $1",
    )
    .bind("google-sub-link")
    .fetch_one(&harness.pool)
    .await
    .expect("oauth link exists");
    assert_eq!(linked_credential_id, credential_id);
}

#[tokio::test]
async fn test_oauth_google_unverified_email_rejected() {
    let harness = TestHarness::new().await;
    harness.google_client.set_user_info(OAuthUserInfo {
        provider_user_id: "google-sub-unverified".to_string(),
        email: "unverified@example.com".to_string(),
        email_verified: false,
        display_name: None,
    });

    let (status, body) = google_login(&harness).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    assert_eq!(body["code"], "OAUTH_EMAIL_NOT_VERIFIED");

    let (credential_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_credentials WHERE email = $1")
            .bind("unverified@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("count credentials");
    assert_eq!(credential_count, 0, "no credential should be created");
}

#[tokio::test]
async fn test_oauth_google_repeat_login_reuses_account() {
    let harness = TestHarness::new().await;
    harness.google_client.set_user_info(OAuthUserInfo {
        provider_user_id: "google-sub-repeat".to_string(),
        email: "repeat@example.com".to_string(),
        email_verified: true,
        display_name: Some("Repeat User".to_string()),
    });

    let (status1, _) = google_login(&harness).await;
    assert_eq!(status1, StatusCode::OK);

    let (status2, body2) = google_login(&harness).await;
    assert_eq!(status2, StatusCode::OK, "body: {body2}");

    // Still exactly one credential and one link.
    let (credential_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM auth_credentials WHERE email = $1")
            .bind("repeat@example.com")
            .fetch_one(&harness.pool)
            .await
            .expect("count credentials");
    assert_eq!(credential_count, 1);

    let (link_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM oauth_accounts WHERE provider = 'google' AND provider_user_id = $1",
    )
    .bind("google-sub-repeat")
    .fetch_one(&harness.pool)
    .await
    .expect("count oauth accounts");
    assert_eq!(link_count, 1);

    // Two sessions (one per login) for the credential.
    let (session_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_sessions s \
         JOIN auth_credentials c ON c.id = s.credential_id \
         WHERE c.email = $1",
    )
    .bind("repeat@example.com")
    .fetch_one(&harness.pool)
    .await
    .expect("count sessions");
    assert_eq!(session_count, 2);
}
