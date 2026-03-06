#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use axum::http::{Method, StatusCode};
use common::helpers::{make_authenticated_request, make_json_request};
use common::setup::TestHarness;
use serde_json::json;
use uuid::Uuid;

// ============================================================
// MESSAGES — send
// ============================================================

#[tokio::test]
async fn send_message_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Hello, world!" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["content"], "Hello, world!");
    assert_eq!(body["channel_id"], channel_id.to_string());
    assert_eq!(body["user_id"], user_id);
    assert_eq!(body["is_deleted"], false);
    assert_eq!(body["message_type"], "text");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn send_message_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let channel_id = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "No auth" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn send_message_empty_content_returns_422() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
}

#[tokio::test]
async fn send_message_too_long_content_returns_422() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();
    let long_content = "a".repeat(2001);

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": long_content })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
}

#[tokio::test]
async fn send_message_with_reply_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    // Send the original message
    let (_, original) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Original message" })),
        &user_id,
    )
    .await;
    let original_id = original["id"].as_str().unwrap();

    // Reply to the original
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({
            "content": "This is a reply",
            "reply_to_id": original_id
        })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["content"], "This is a reply");
    assert_eq!(body["reply_to_id"], original_id);
}

// ============================================================
// MESSAGES — get
// ============================================================

#[tokio::test]
async fn get_messages_empty_channel_returns_200() {
    let harness = TestHarness::new().await;
    let channel_id = Uuid::new_v4();

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"].as_array().unwrap().len(), 0);
    assert_eq!(body["has_more"], false);
}

#[tokio::test]
async fn get_messages_returns_sent_messages() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    for i in 1..=3u32 {
        make_authenticated_request(
            harness.router.clone(),
            Method::POST,
            &format!("/api/v1/channels/{channel_id}/messages"),
            Some(json!({ "content": format!("Message {i}") })),
            &user_id,
        )
        .await;
    }

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"].as_array().unwrap().len(), 3);
    assert_eq!(body["has_more"], false);
}

#[tokio::test]
async fn get_messages_respects_limit() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    for i in 1..=5u32 {
        make_authenticated_request(
            harness.router.clone(),
            Method::POST,
            &format!("/api/v1/channels/{channel_id}/messages"),
            Some(json!({ "content": format!("Message {i}") })),
            &user_id,
        )
        .await;
    }

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages?limit=2"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(body["has_more"], true);
}

#[tokio::test]
async fn get_messages_with_before_pagination() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    // Send 3 messages and collect their IDs (newest last due to DESC order in response)
    let mut ids = Vec::new();
    for i in 1..=3u32 {
        let (_, body) = make_authenticated_request(
            harness.router.clone(),
            Method::POST,
            &format!("/api/v1/channels/{channel_id}/messages"),
            Some(json!({ "content": format!("Msg {i}") })),
            &user_id,
        )
        .await;
        ids.push(body["id"].as_str().unwrap().to_owned());
    }

    // The third message is the newest; use its ID as the `before` cursor
    let before_id = &ids[2];
    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages?before={before_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let messages = body["messages"].as_array().unwrap();
    // Should return the two earlier messages
    assert_eq!(messages.len(), 2);
    // None of them should be the `before` message itself
    assert!(messages.iter().all(|m| m["id"] != *before_id));
}

// ============================================================
// MESSAGES — edit
// ============================================================

#[tokio::test]
async fn edit_message_by_author_returns_200() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Original" })),
        &user_id,
    )
    .await;
    let message_id = created["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{message_id}"),
        Some(json!({ "content": "Edited content" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"], "Edited content");
    assert_eq!(body["id"], message_id);
    assert!(body["edited_at"].is_string());
}

#[tokio::test]
async fn edit_message_by_non_author_returns_403() {
    let harness = TestHarness::new().await;
    let author_id = Uuid::new_v4().to_string();
    let other_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Author's message" })),
        &author_id,
    )
    .await;
    let message_id = created["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{message_id}"),
        Some(json!({ "content": "Hijacked edit" })),
        &other_id,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn edit_message_not_found_returns_404() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let nonexistent = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{nonexistent}"),
        Some(json!({ "content": "Edit ghost" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn edit_message_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let nonexistent = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{nonexistent}"),
        Some(json!({ "content": "No auth edit" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================
// MESSAGES — delete
// ============================================================

#[tokio::test]
async fn delete_message_by_author_returns_204() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "To be deleted" })),
        &user_id,
    )
    .await;
    let message_id = created["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/messages/{message_id}"),
        None,
        &user_id,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Deleted message should no longer appear in the channel listing
    let (_, list_body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages"),
        None,
    )
    .await;
    assert_eq!(list_body["messages"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_message_by_non_author_returns_403() {
    let harness = TestHarness::new().await;
    let author_id = Uuid::new_v4().to_string();
    let other_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Protected" })),
        &author_id,
    )
    .await;
    let message_id = created["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{message_id}"),
        None,
        &other_id,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn delete_message_not_found_returns_404() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let nonexistent = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{nonexistent}"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn delete_message_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let nonexistent = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{nonexistent}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================
// REACTIONS — add
// ============================================================

#[tokio::test]
async fn add_reaction_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React to me" })),
        &user_id,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/thumbsup"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["user_id"], user_id);
    assert_eq!(body["emoji"], "thumbsup");
}

#[tokio::test]
async fn add_reaction_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let nonexistent = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{nonexistent}/reactions/heart"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn add_reaction_to_nonexistent_message_returns_404() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let nonexistent = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{nonexistent}/reactions/thumbsup"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn add_duplicate_reaction_returns_409() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Double react" })),
        &user_id,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/fire"),
        None,
        &user_id,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/fire"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "conflict");
}

// ============================================================
// REACTIONS — get
// ============================================================

#[tokio::test]
async fn get_reactions_returns_list() {
    let harness = TestHarness::new().await;
    let user1 = Uuid::new_v4().to_string();
    let user2 = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Many reacts" })),
        &user1,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/heart"),
        None,
        &user1,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/heart"),
        None,
        &user2,
    )
    .await;

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/messages/{message_id}/reactions"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let reactions = body.as_array().unwrap();
    assert_eq!(reactions.len(), 2);
    assert!(reactions.iter().all(|r| r["emoji"] == "heart"));
}

#[tokio::test]
async fn get_reactions_empty_returns_empty_list() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "No reactions yet" })),
        &user_id,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/messages/{message_id}/reactions"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ============================================================
// REACTIONS — remove
// ============================================================

#[tokio::test]
async fn remove_reaction_returns_204() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React then remove" })),
        &user_id,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{message_id}/reactions/wave"),
        None,
        &user_id,
    )
    .await;

    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/messages/{message_id}/reactions/wave"),
        None,
        &user_id,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (_, list_body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/messages/{message_id}/reactions"),
        None,
    )
    .await;
    assert_eq!(list_body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn remove_nonexistent_reaction_returns_404() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "No reaction on this" })),
        &user_id,
    )
    .await;
    let message_id = msg["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{message_id}/reactions/ghost"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn remove_reaction_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let nonexistent = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{nonexistent}/reactions/heart"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
