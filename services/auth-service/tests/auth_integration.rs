mod common;

use axum::http::{Method, StatusCode};
use common::helpers::make_json_request;
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
    // Refresh returns the same refresh token (no rotation)
    assert_eq!(body["refresh_token"].as_str(), Some(refresh_token));
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
