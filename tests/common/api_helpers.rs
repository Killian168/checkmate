use crate::common::models::*;
use crate::common::utils::*;
use reqwest::{Client, Response};
use std::time::Duration;

pub fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client")
}

pub async fn create_user(client: &Client, email: &str, password: &str) -> Response {
    let url = format!("{}/auth/user", base_url());
    let payload = CreateUserPayload {
        email,
        password,
        first_name: "Integration",
        last_name: "Tester",
    };
    client
        .post(url)
        .json(&payload)
        .send()
        .await
        .expect("Failed to send create_user request")
}

pub async fn login(client: &Client, email: &str, password: &str) -> Response {
    let url = format!("{}/auth/login", base_url());
    let payload = LoginPayload { email, password };
    client
        .post(url)
        .json(&payload)
        .send()
        .await
        .expect("Failed to send login request")
}

pub async fn get_user(client: &Client, token: Option<&str>) -> Response {
    let url = format!("{}/auth/user", base_url());
    let mut req = client.get(url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }
    req.send().await.expect("Failed to send get_user request")
}

pub async fn delete_user(client: &Client, token: &str) -> Response {
    let url = format!("{}/auth/user", base_url());
    client
        .delete(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send delete_user request")
}

pub async fn get_jwt_from_login(resp: Response) -> String {
    assert!(
        resp.status().is_success(),
        "Expected login success, got {}",
        resp.status()
    );
    let body: LoginResponse = resp.json().await.expect("Invalid login response JSON");
    assert!(!body.token.is_empty(), "JWT token was empty");
    body.token
}
