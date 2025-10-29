use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json;
use shared::User;
use std::env;
use std::time::{Duration, SystemTime};

use super::cognito_auth::{
    authenticate_with_cognito, create_test_cognito_user, create_test_dynamodb_user,
    delete_cognito_user, get_test_auth_token,
};
use super::load_env;

fn get_api_url() -> String {
    load_env();
    env::var("API_URL").unwrap_or_else(|_| panic!("API_URL environment variable not set."))
}

async fn create_authenticated_client() -> (Client, String) {
    let client = Client::new();
    let token = get_test_auth_token().await;
    (client, token)
}

#[tokio::test]
async fn e2e_users_me_endpoint_returns_user_data() {
    let api_url = get_api_url();
    let (client, token) = create_authenticated_client().await;

    let response = client
        .get(format!("{}/users/me", api_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to /users/me endpoint");

    let status = response.status();
    assert_eq!(status, 200);

    let user: User = response
        .json()
        .await
        .expect("Failed to parse response as User");
    assert_eq!(user.rating, 1200);
}

#[tokio::test]
async fn e2e_users_me_endpoint_returns_401_when_unauthorized() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/users/me", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to /users/me endpoint");

    let status = response.status();
    assert_eq!(status, 401);
}

#[tokio::test]
async fn e2e_users_me_endpoint_delete_user() {
    let api_url = get_api_url();

    // Generate unique email for test user
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_email = format!("test-delete-{timestamp}@example.com");
    let test_password = "TempPassword123!";

    // Ensure cleanup happens even if test fails
    let result = async {
        // Create a new Cognito user
        create_test_cognito_user(&test_email, &test_password)
            .await
            .expect("Failed to create test Cognito user");

        // Authenticate with the new user
        let tokens = authenticate_with_cognito(&test_email, &test_password)
            .await
            .expect("Failed to authenticate with test user");

        // Decode the JWT to extract the user_id (sub)
        let jwt_parts: Vec<&str> = tokens.id_token.split('.').collect();
        assert_eq!(jwt_parts.len(), 3, "Invalid JWT format");
        let payload = general_purpose::URL_SAFE_NO_PAD
            .decode(jwt_parts[1])
            .expect("Invalid JWT payload");
        let claims: serde_json::Value =
            serde_json::from_slice(&payload).expect("Invalid claims JSON");
        let user_id = claims["sub"]
            .as_str()
            .expect("Missing sub in JWT claims")
            .to_string();

        // Debug logging

        // Create DynamoDB entry for the user
        create_test_dynamodb_user(&user_id)
            .await
            .expect("Failed to create test DynamoDB user entry");

        let client = Client::new();

        // Delete the user account using the API
        let response = client
            .delete(format!("{}/users/me", api_url))
            .header("Authorization", format!("Bearer {}", tokens.id_token))
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .expect("Failed to send DELETE request to /users/me endpoint");

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        assert_eq!(
            status, 204,
            "Expected 204 No Content on successful deletion, got {}: {}",
            status, response_text
        );
    }
    .await;

    // Always attempt to clean up, regardless of test success/failure
    let _ = delete_cognito_user(&test_email).await;

    // Propagate any test error after cleanup
    result
}

#[tokio::test]
async fn e2e_users_me_delete_endpoint_returns_401_when_unauthorized() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .delete(format!("{}/users/me", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send DELETE request to /users/me endpoint");

    let status = response.status();
    assert_eq!(status, 401);
}
