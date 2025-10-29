use reqwest::Client;
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
async fn e2e_health_endpoint_returns_200() {
    let api_url = get_api_url();
    let (client, _token) = create_authenticated_client().await;

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to health endpoint");

    let status = response.status();
    assert_eq!(status, 200);
}
