mod common;

use axum::http::{Method, StatusCode};
use common::helpers::make_json_request;
use common::setup::TestHarness;
use serde_json::json;

// ============================================
// HELPER: create a profile and return (status, body)
// ============================================
async fn create_profile(
    harness: &TestHarness,
    username: &str,
    display_name: &str,
) -> (StatusCode, serde_json::Value) {
    // The route is POST /api/users/users/:user_id but the handler
    // ignores the path param; the ID is auto-generated.
    let dummy_id = uuid::Uuid::new_v4();
    make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/users/users/{dummy_id}"),
        Some(json!({
            "username": username,
            "display_name": display_name
        })),
    )
    .await
}

// ============================================
// PROFILE MANAGEMENT TESTS
// ============================================

#[tokio::test]
async fn test_create_profile_success() {
    let harness = TestHarness::new().await;

    let (status, body) = create_profile(&harness, "alice", "Alice").await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["username"], "alice");
    assert_eq!(body["display_name"], "Alice");
    assert!(body["user_id"].is_string());
}

#[tokio::test]
async fn test_create_profile_invalid_username() {
    let harness = TestHarness::new().await;

    // Uppercase not allowed
    let (status, _) = create_profile(&harness, "Alice", "Alice").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Too short (< 3 chars)
    let (status, _) = create_profile(&harness, "ab", "AB").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_create_profile_duplicate_username() {
    let harness = TestHarness::new().await;

    let (status, _) = create_profile(&harness, "dupuser", "Dup One").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = create_profile(&harness, "dupuser", "Dup Two").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ============================================
// GET PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_get_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "getme", "Get Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/users/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "getme");
    assert_eq!(body["display_name"], "Get Me");
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let harness = TestHarness::new().await;
    let random_id = uuid::Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/users/users/{random_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_profile_by_username() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "byname", "By Name").await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/users/users/username/byname",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "byname");
}

// ============================================
// UPDATE PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_update_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "updateme", "Update Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PATCH,
        &format!("/api/users/users/{user_id}"),
        Some(json!({
            "display_name": "Updated Name",
            "bio": "A new bio"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["display_name"], "Updated Name");
    assert_eq!(body["bio"], "A new bio");
}

#[tokio::test]
async fn test_update_profile_not_found() {
    let harness = TestHarness::new().await;
    let random_id = uuid::Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PATCH,
        &format!("/api/users/users/{random_id}"),
        Some(json!({ "display_name": "Ghost" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// CHANGE USERNAME TESTS
// ============================================

#[tokio::test]
async fn test_change_username_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "oldname", "Old Name").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/username"),
        Some(json!({ "new_username": "newname" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "newname");
}

#[tokio::test]
async fn test_change_username_already_taken() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "taken_name", "Taken").await;
    let (_, create_body) = create_profile(&harness, "wants_taken", "Wants Taken").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/username"),
        Some(json!({ "new_username": "taken_name" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

// ============================================
// DELETE PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_delete_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "deleteme", "Delete Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/users/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_then_get() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "delete_get", "Delete Get").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    // Delete
    make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/users/users/{user_id}"),
        None,
    )
    .await;

    // Get should now 404
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/users/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// STATUS & PRESENCE TESTS
// ============================================

#[tokio::test]
async fn test_update_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "statususer", "Status User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    for status_val in &["online", "idle", "dnd", "offline"] {
        let (status, body) = make_json_request(
            harness.router.clone(),
            Method::PUT,
            &format!("/api/users/users/{user_id}/status"),
            Some(json!({ "status": status_val })),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "setting {status_val} failed: {body}");
        assert_eq!(body["status"], *status_val);
    }
}

#[tokio::test]
async fn test_set_custom_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "customstatus", "Custom Status").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/custom-status"),
        Some(json!({
            "text": "Working hard",
            "emoji": "hammer"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["custom_status"]["text"], "Working hard");
    assert_eq!(body["custom_status"]["emoji"], "hammer");
}

#[tokio::test]
async fn test_clear_custom_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "clearstatus", "Clear Status").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    // Set a custom status first
    make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/custom-status"),
        Some(json!({ "text": "Temporary" })),
    )
    .await;

    // Clear it
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/users/users/{user_id}/custom-status"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================
// SEARCH & DISCOVERY TESTS
// ============================================

#[tokio::test]
async fn test_check_username_availability() {
    let harness = TestHarness::new().await;

    // Available
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/users/users/check-username/available_name",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["available"], true);

    // Take it
    create_profile(&harness, "available_name", "Taken Now").await;

    // No longer available
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/users/users/check-username/available_name",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["available"], false);
}

#[tokio::test]
async fn test_search_users() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "searchable_alice", "Alice Searchable").await;
    create_profile(&harness, "searchable_bob", "Bob Searchable").await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/users/users/search?query=searchable&limit=10&offset=0",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let profiles = body["profiles"].as_array().expect("profiles array");
    assert!(
        profiles.len() >= 2,
        "expected at least 2 results, got {}",
        profiles.len()
    );
}

// ============================================
// PRIVACY TESTS
// ============================================

#[tokio::test]
async fn test_get_privacy_defaults() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "privacyuser", "Privacy User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/users/users/{user_id}/privacy"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    // Check defaults from the migration
    assert_eq!(body["allow_dms_from"], "friends");
    assert_eq!(body["allow_friend_requests_from"], "everyone");
    assert_eq!(body["show_online_status"], true);
    assert_eq!(body["show_current_activity"], true);
    assert_eq!(body["show_profile_to_non_friends"], true);
    assert_eq!(body["allow_nsfw_content"], false);
    assert_eq!(body["content_filter_level"], 1);
}

#[tokio::test]
async fn test_update_dm_privacy() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "dmuser", "DM User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/privacy/dm"),
        Some(json!({ "allow_dms_from": "none" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_dms_from"], "none");
}

#[tokio::test]
async fn test_update_friend_request_privacy() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "fruser", "FR User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/users/users/{user_id}/privacy/friend-requests"),
        Some(json!({ "allow_friend_requests_from": "none" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_friend_requests_from"], "none");
}

#[tokio::test]
async fn test_apply_privacy_preset() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "presetuser", "Preset User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/users/users/{user_id}/privacy/preset"),
        Some(json!({ "preset": "private" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    // Private preset should restrict most settings
    assert_eq!(body["allow_dms_from"], "none");
    assert_eq!(body["allow_friend_requests_from"], "none");
    assert_eq!(body["show_online_status"], false);
    assert_eq!(body["show_current_activity"], false);
    assert_eq!(body["show_profile_to_non_friends"], false);
}
