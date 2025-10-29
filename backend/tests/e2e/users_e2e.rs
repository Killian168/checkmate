use reqwest::Client;
use shared::User;
use std::env;
use std::time::Duration;

use super::cognito_auth::get_test_auth_token;
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
    // Assuming the test user has some data; adjust based on test setup
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
