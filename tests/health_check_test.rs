pub mod common;
use crate::common::utils::*;
use reqwest::StatusCode;

#[tokio::test]
async fn test_health_endpoint() {
    let client = reqwest::Client::new();
    let url = format!("{}/health", base_url());
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send /health request");
    assert_eq!(resp.status(), StatusCode::OK);
}
