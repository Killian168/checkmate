pub mod common;
use common::api_helpers::*;
use common::models::*;
use common::utils::*;
use reqwest::StatusCode;

/// Test the full success flow: create user, login, get user info, delete user, token invalid after deletion.
#[tokio::test]
async fn test_auth_flow_success() {
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

    // 3) Get user info
    let user_resp = get_user(&client, Some(&jwt)).await;
    assert!(
        user_resp.status().is_success(),
        "Expected successful GET /auth/user, got {}",
        user_resp.status()
    );

    let user: UserResponse = user_resp
        .json()
        .await
        .expect("Failed to parse /auth/user JSON; response may be malformed");

    assert_eq!(
        user.email, email,
        "Returned user email does not match created user"
    );
    assert_eq!(
        user.first_name, "Integration",
        "Returned firstName does not match"
    );
    assert_eq!(user.last_name, "Tester", "Returned lastName does not match");
    assert!(!user.id.is_empty(), "Returned user ID should not be empty");
    assert!(
        !user.created_at.is_empty(),
        "Returned user createdAt timestamp should not be empty"
    );

    // 4) Delete user
    let del_resp = delete_user(&client, &jwt).await;
    assert!(
        del_resp.status() == StatusCode::OK || del_resp.status() == StatusCode::NO_CONTENT,
        "Expected 200 OK or 204 No Content for DELETE /auth/user, got {}",
        del_resp.status()
    );
}

/// Test that invalid login credentials are rejected.
#[tokio::test]
async fn test_invalid_login_rejected() {
    let client = http_client();
    let email = random_email();
    let password = test_password();

    // Create user
    let create_resp = create_user(&client, &email, &password).await;
    assert_eq!(
        create_resp.status(),
        StatusCode::CREATED,
        "Failed to create user for invalid login test"
    );

    // Attempt login with wrong password
    let bad_login_resp = login(&client, &email, "wrong-password").await;
    assert_eq!(
        bad_login_resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized for login with incorrect password, got {}",
        bad_login_resp.status()
    );
}

/// Test that accessing /auth/user without a token returns 401
#[tokio::test]
async fn test_get_user_without_token_unauthorized() {
    let client = http_client();
    let resp = get_user(&client, None).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /auth/user without a token, got {}",
        resp.status()
    );
}

/// Test that accessing /auth/user with an invalid token returns 401
#[tokio::test]
async fn test_get_user_with_invalid_token_unauthorized() {
    let client = http_client();
    let resp = get_user(&client, Some("invalid.jwt.token")).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /auth/user with an invalid token, got {}",
        resp.status()
    );
}
