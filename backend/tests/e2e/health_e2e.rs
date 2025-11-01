use reqwest::Client;
use std::env;
use std::time::Duration;

use super::load_env;

fn get_api_url() -> String {
    load_env();
    env::var("API_URL").unwrap_or_else(|_| panic!("API_URL environment variable not set."))
}

#[tokio::test]
async fn e2e_health_endpoint_returns_200() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to health endpoint");

    let status = response.status();
    assert_eq!(status, 200);
}
