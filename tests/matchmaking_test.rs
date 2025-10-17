pub mod common;
use common::api_helpers::*;

use common::utils::*;
use reqwest::StatusCode;
use serde_json::json;

/// Test the full matchmaking flow: create user, join queue, leave queue
#[tokio::test]
async fn test_matchmaking_flow_success() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // 1) Create user
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(
        create_resp.status(),
        StatusCode::CREATED,
        "Expected 201 Created when creating user, got {}",
        create_resp.status()
    );

    // 2) Login
    let login_resp = login(&client, &email, &password).await;
    let jwt = get_jwt_from_login(login_resp).await;

    // 3) Join matchmaking queue
    let join_resp = join_queue(&client, &jwt, "competitive").await;
    assert_eq!(
        join_resp.status(),
        StatusCode::OK,
        "Expected 200 OK when joining queue, got {}",
        join_resp.status()
    );

    // 4) Leave matchmaking queue
    let leave_resp = leave_queue(&client, &jwt, "competitive").await;
    assert_eq!(
        leave_resp.status(),
        StatusCode::OK,
        "Expected 200 OK when leaving queue, got {}",
        leave_resp.status()
    );

    // 5) Clean up - delete user
    let del_resp = delete_user(&client, &jwt).await;
    assert!(
        del_resp.status() == StatusCode::OK || del_resp.status() == StatusCode::NO_CONTENT,
        "Expected 200 OK or 204 No Content for DELETE /auth/user, got {}",
        del_resp.status()
    );
}

/// Test joining matchmaking queue without authentication
#[tokio::test]
async fn test_join_queue_unauthorized() {
    let client = http_client();
    let join_resp = join_queue(&client, "", "competitive").await;
    assert_eq!(
        join_resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when joining queue without auth, got {}",
        join_resp.status()
    );
}

/// Test leaving matchmaking queue without authentication
#[tokio::test]
async fn test_leave_queue_unauthorized() {
    let client = http_client();
    let leave_resp = leave_queue(&client, "", "competitive").await;
    assert_eq!(
        leave_resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when leaving queue without auth, got {}",
        leave_resp.status()
    );
}

/// Test joining matchmaking queue with invalid JWT
#[tokio::test]
async fn test_join_queue_invalid_token() {
    let client = http_client();
    let join_resp = join_queue(&client, "invalid.jwt.token", "competitive").await;
    assert_eq!(
        join_resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when joining queue with invalid token, got {}",
        join_resp.status()
    );
}

/// Test joining multiple queue types
#[tokio::test]
async fn test_join_different_queue_types() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // Create user and login
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let login_resp = login(&client, &email, &password).await;
    let jwt = get_jwt_from_login(login_resp).await;

    // Test joining different queue types
    let queue_types = vec!["competitive", "casual", "tournament"];

    for queue_type in queue_types {
        let join_resp = join_queue(&client, &jwt, queue_type).await;
        assert_eq!(
            join_resp.status(),
            StatusCode::OK,
            "Expected 200 OK when joining {} queue, got {}",
            queue_type,
            join_resp.status()
        );

        let leave_resp = leave_queue(&client, &jwt, queue_type).await;
        assert_eq!(
            leave_resp.status(),
            StatusCode::OK,
            "Expected 200 OK when leaving {} queue, got {}",
            queue_type,
            leave_resp.status()
        );
    }

    // Clean up
    let del_resp = delete_user(&client, &jwt).await;
    assert!(del_resp.status().is_success());
}

/// Test joining queue with empty queue type
#[tokio::test]
async fn test_join_queue_empty_type() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // Create user and login
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let login_resp = login(&client, &email, &password).await;
    let jwt = get_jwt_from_login(login_resp).await;

    // Try to join with empty queue type
    let join_resp = join_queue(&client, &jwt, "").await;
    // This should either return 400 Bad Request or handle it gracefully
    // The exact behavior depends on the validation logic
    assert!(
        join_resp.status() == StatusCode::BAD_REQUEST || join_resp.status() == StatusCode::OK,
        "Expected 400 Bad Request or 200 OK for empty queue type, got {}",
        join_resp.status()
    );

    // Clean up
    let del_resp = delete_user(&client, &jwt).await;
    assert!(del_resp.status().is_success());
}

/// Test leaving queue that user hasn't joined
#[tokio::test]
async fn test_leave_queue_not_joined() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // Create user and login
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let login_resp = login(&client, &email, &password).await;
    let jwt = get_jwt_from_login(login_resp).await;

    // Try to leave a queue without joining first
    let leave_resp = leave_queue(&client, &jwt, "competitive").await;
    // This should either return 400 Bad Request or handle it gracefully
    assert!(
        leave_resp.status() == StatusCode::BAD_REQUEST || leave_resp.status() == StatusCode::OK,
        "Expected 400 Bad Request or 200 OK when leaving unjoined queue, got {}",
        leave_resp.status()
    );

    // Clean up
    let del_resp = delete_user(&client, &jwt).await;
    assert!(del_resp.status().is_success());
}

/// Test concurrent queue operations
#[tokio::test]
async fn test_concurrent_queue_operations() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // Create user and login
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let login_resp = login(&client, &email, &password).await;
    let jwt = get_jwt_from_login(login_resp).await;

    // Join queue multiple times (should handle gracefully)
    let join_resp1 = join_queue(&client, &jwt, "competitive").await;
    let _join_resp2 = join_queue(&client, &jwt, "competitive").await;

    // Both operations should succeed or the second one might return an error
    // depending on whether duplicate joins are allowed
    assert!(
        join_resp1.status().is_success(),
        "First join should succeed, got {}",
        join_resp1.status()
    );

    // Leave queue
    let leave_resp = leave_queue(&client, &jwt, "competitive").await;
    assert!(
        leave_resp.status().is_success(),
        "Leave should succeed, got {}",
        leave_resp.status()
    );

    // Clean up
    let del_resp = delete_user(&client, &jwt).await;
    assert!(del_resp.status().is_success());
}

// Helper functions for matchmaking API calls
pub async fn join_queue(
    client: &reqwest::Client,
    token: &str,
    queue_type: &str,
) -> reqwest::Response {
    let url = format!("{}/matchmaking/join", base_url());
    let payload = json!({
        "queue_type": queue_type
    });

    let mut req = client.post(url).json(&payload);
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    req.send().await.expect("Failed to send join_queue request")
}

pub async fn leave_queue(
    client: &reqwest::Client,
    token: &str,
    queue_type: &str,
) -> reqwest::Response {
    let url = format!("{}/matchmaking/leave", base_url());
    let payload = json!({
        "queue_type": queue_type
    });

    let mut req = client.post(url).json(&payload);
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    req.send()
        .await
        .expect("Failed to send leave_queue request")
}
