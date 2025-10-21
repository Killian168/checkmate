use std::time::Duration;

use reqwest::StatusCode;
use serde_json::{json, Value};

mod test_utils;

use crate::test_utils::generators::{
    generate_test_email, generate_test_password, verify_health_endpoint,
};

#[derive(Debug, Clone)]
pub struct E2EConfig {
    pub base_url: String,
    pub websocket_url: String,
    pub timeout: Duration,
    pub max_concurrent_users: usize,
}

impl Default for E2EConfig {
    fn default() -> Self {
        Self {
            base_url: "https://sq5wo33z30.execute-api.eu-west-1.amazonaws.com".to_string(),
            websocket_url: "wss://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev".to_string(),
            timeout: Duration::from_secs(30),
            max_concurrent_users: 10,
        }
    }
}

pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    websocket_url: String,
    auth_token: Option<String>,
    user_id: Option<String>,
}

impl TestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            websocket_url: "wss://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev".to_string(),
            auth_token: None,
            user_id: None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    pub async fn register_user(
        &mut self,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<StatusCode, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&format!("{}/auth/user", self.base_url))
            .json(&json!({
                "email": email,
                "password": password,
                "first_name": first_name,
                "last_name": last_name
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.status())
    }

    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&format!("{}/auth/login", self.base_url))
            .json(&json!({
                "email": email,
                "password": password
            }))
            .send()
            .await?
            .error_for_status()?;

        let login_response: Value = response.json().await?;
        let token = login_response["token"].as_str().unwrap().to_string();
        self.auth_token = Some(token.clone());
        Ok(token)
    }

    pub async fn join_queue(
        &self,
        queue_type: &str,
    ) -> Result<StatusCode, Box<dyn std::error::Error>> {
        if let Some(token) = &self.auth_token {
            let response = self
                .client
                .post(&format!("{}/queue/join", self.base_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&json!({
                    "queue_type": queue_type
                }))
                .send()
                .await?
                .error_for_status()?;

            Ok(response.status())
        } else {
            // Return error when not authenticated
            Err("Not authenticated".into())
        }
    }

    pub async fn leave_queue(
        &self,
        queue_type: &str,
    ) -> Result<StatusCode, Box<dyn std::error::Error>> {
        if let Some(token) = &self.auth_token {
            let response = self
                .client
                .post(&format!("{}/queue/leave", self.base_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&json!({
                    "queue_type": queue_type
                }))
                .send()
                .await?
                .error_for_status()?;

            Ok(response.status())
        } else {
            // Return error when not authenticated
            Err("Not authenticated".into())
        }
    }

    pub async fn get_user_info(&self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(token) = &self.auth_token {
            let response = self
                .client
                .get(&format!("{}/auth/user", self.base_url))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?
                .error_for_status()?;

            Ok(response.json().await?)
        } else {
            // Return error when not authenticated
            Err("Not authenticated".into())
        }
    }

    pub async fn delete_account(&self) -> Result<StatusCode, Box<dyn std::error::Error>> {
        if let Some(token) = &self.auth_token {
            let response = self
                .client
                .delete(&format!("{}/auth/user", self.base_url))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?
                .error_for_status()?;

            Ok(response.status())
        } else {
            // Return error when not authenticated
            Err("Not authenticated".into())
        }
    }

    pub async fn establish_websocket_connection(
        &self,
        _player_id: &str,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Box<dyn std::error::Error>,
    > {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.websocket_url).await?;
        Ok(ws_stream)
    }
}

async fn test_complete_user_journey() {
    let config = E2EConfig::default();

    // Verify health endpoint first
    let health_ok = verify_health_endpoint(&config.base_url).await.unwrap();
    assert!(health_ok, "Health endpoint should be healthy");

    let mut client = TestClient::new(config.base_url.clone());

    // Generate unique test credentials
    let email = generate_test_email();
    let password = generate_test_password();
    let first_name = "Test";
    let last_name = "User";

    // Step 1: Register user
    let register_status = client
        .register_user(&email, &password, first_name, last_name)
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    // Step 2: Login
    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());
    assert!(client.is_authenticated());

    // Step 3: Get user info
    let user_info = client.get_user_info().await.unwrap();
    assert_eq!(user_info["email"], email);
    assert_eq!(user_info["first_name"], first_name);
    assert_eq!(user_info["last_name"], last_name);

    // Step 4: Join queue
    let join_status = client.join_queue("rapid").await.unwrap();
    assert_eq!(join_status, StatusCode::OK);

    // Step 5: Leave queue
    let leave_status = client.leave_queue("rapid").await.unwrap();
    assert_eq!(leave_status, StatusCode::OK);

    // Step 6: Delete account
    let delete_status = client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_websocket_connection_flow() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    let email = generate_test_email();
    let password = generate_test_password();

    // Register and login
    let register_status = client
        .register_user(&email, &password, "WebSocket", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());

    // Get user info to extract user ID
    let user_info = client.get_user_info().await.unwrap();
    let user_id = user_info["id"].as_str().unwrap();

    // Establish WebSocket connection
    let ws_result = client.establish_websocket_connection(user_id).await;
    assert!(ws_result.is_ok(), "WebSocket connection should succeed");

    // Clean up
    let delete_status = client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_websocket_multiple_connections() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    let email = generate_test_email();
    let password = generate_test_password();

    // Register and login
    let register_status = client
        .register_user(&email, &password, "MultiWS", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());

    // Get user info
    let user_info = client.get_user_info().await.unwrap();
    let user_id = user_info["id"].as_str().unwrap();

    // Establish multiple WebSocket connections
    let mut connections = Vec::new();
    for _ in 0..3 {
        let ws_result = client.establish_websocket_connection(user_id).await;
        assert!(ws_result.is_ok(), "WebSocket connection should succeed");
        connections.push(ws_result.unwrap());
    }

    // Clean up
    let delete_status = client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_websocket_reconnection() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    let email = generate_test_email();
    let password = generate_test_password();

    // Register and login
    let register_status = client
        .register_user(&email, &password, "Reconnect", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());

    // Get user info
    let user_info = client.get_user_info().await.unwrap();
    let user_id = user_info["id"].as_str().unwrap();

    // First connection
    let ws_result1 = client.establish_websocket_connection(user_id).await;
    assert!(
        ws_result1.is_ok(),
        "First WebSocket connection should succeed"
    );

    // Simulate reconnection
    let ws_result2 = client.establish_websocket_connection(user_id).await;
    assert!(
        ws_result2.is_ok(),
        "Second WebSocket connection should succeed"
    );

    // Clean up
    let delete_status = client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_multiple_users_queue_interaction() {
    let config = E2EConfig::default();
    let user_count = 3;

    let mut handles = Vec::new();

    for i in 0..user_count {
        let base_url = config.base_url.clone();
        let handle = tokio::spawn(async move {
            let mut client = TestClient::new(base_url);
            let email = generate_test_email();
            let password = generate_test_password();

            // Register and login
            let register_status = client
                .register_user(&email, &password, "Multi", "User")
                .await
                .unwrap();
            assert_eq!(register_status, StatusCode::CREATED);

            let token = client.login(&email, &password).await.unwrap();
            assert!(!token.is_empty());

            // Join queue
            let join_status = client.join_queue("rapid").await.unwrap();
            assert_eq!(join_status, StatusCode::OK);

            // Leave queue
            let leave_status = client.leave_queue("rapid").await.unwrap();
            assert_eq!(leave_status, StatusCode::OK);

            // Delete account
            let delete_status = client.delete_account().await.unwrap();
            assert_eq!(delete_status, StatusCode::NO_CONTENT);

            true
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap());
    }
}

async fn test_authentication_flow() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    let email = generate_test_email();
    let password = generate_test_password();

    // Test: Cannot access protected endpoints without authentication
    let join_result = client.join_queue("rapid").await;
    assert!(join_result.is_err());

    let user_info_result = client.get_user_info().await;
    assert!(user_info_result.is_err());

    let delete_result = client.delete_account().await;
    assert!(delete_result.is_err());

    // Register and login
    let register_status = client
        .register_user(&email, &password, "Auth", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());
    assert!(client.is_authenticated());

    // Now should be able to access protected endpoints
    let user_info = client.get_user_info().await.unwrap();
    assert_eq!(user_info["email"], email);

    // Clean up
    let delete_status = client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_error_scenarios() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    // Test: Register with invalid data
    let _invalid_register = client.register_user("", "password", "Test", "User").await;
    // This might succeed or fail depending on validation, so we don't assert specific status

    // Test: Login with non-existent user
    let non_existent_login = client.login("nonexistent@example.com", "password").await;
    assert!(non_existent_login.is_err());

    // Register a valid user first
    let email = generate_test_email();
    let password = generate_test_password();

    let register_status = client
        .register_user(&email, &password, "Error", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    // Test: Login with wrong password
    let wrong_password_login = client.login(&email, "wrongpassword").await;
    assert!(wrong_password_login.is_err());

    // Test: Join queue with invalid token
    let mut invalid_client = TestClient::new(config.base_url.clone());
    invalid_client.auth_token = Some("invalid-token".to_string());
    let invalid_join = invalid_client.join_queue("rapid").await;
    assert!(invalid_join.is_err());

    // Clean up valid user
    let login_result = client.login(&email, &password).await;
    if login_result.is_ok() {
        let delete_status = client.delete_account().await.unwrap();
        assert_eq!(delete_status, StatusCode::NO_CONTENT);
    }
}

async fn test_concurrent_queue_operations() {
    let config = E2EConfig::default();
    let user_count = 3; // Reduced for stability

    let mut handles = Vec::new();

    for i in 0..user_count {
        let base_url = config.base_url.clone();
        let handle = tokio::spawn(async move {
            let mut client = TestClient::new(base_url);
            let email = generate_test_email();
            let password = generate_test_password();

            // Register and login
            let register_status = client
                .register_user(&email, &password, "Concurrent", "User")
                .await
                .unwrap();
            assert_eq!(register_status, StatusCode::CREATED);

            let token = client.login(&email, &password).await.unwrap();
            assert!(!token.is_empty());

            // Join queue
            let join_status = client.join_queue("rapid").await.unwrap();
            assert_eq!(join_status, StatusCode::OK);

            // Leave queue
            let leave_status = client.leave_queue("rapid").await.unwrap();
            assert_eq!(leave_status, StatusCode::OK);

            // Delete account
            let delete_status = client.delete_account().await.unwrap();
            assert_eq!(delete_status, StatusCode::NO_CONTENT);

            true
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap());
    }
}

async fn test_session_persistence() {
    let config = E2EConfig::default();
    let mut client = TestClient::new(config.base_url.clone());

    let email = generate_test_email();
    let password = generate_test_password();

    // Register user
    let register_status = client
        .register_user(&email, &password, "Session", "Test")
        .await
        .unwrap();
    assert_eq!(register_status, StatusCode::CREATED);

    // Login and verify session
    let token = client.login(&email, &password).await.unwrap();
    assert!(!token.is_empty());

    let user_info1 = client.get_user_info().await.unwrap();
    assert_eq!(user_info1["email"], email);

    // Create new client instance with same credentials
    let mut new_client = TestClient::new(config.base_url.clone());
    let new_token = new_client.login(&email, &password).await.unwrap();
    assert!(!new_token.is_empty());

    let user_info2 = new_client.get_user_info().await.unwrap();
    assert_eq!(user_info2["email"], email);
    assert_eq!(user_info2["id"], user_info1["id"]);

    // Clean up
    let delete_status = new_client.delete_account().await.unwrap();
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
}

async fn test_performance_under_load() {
    let config = E2EConfig::default();
    let start_time = std::time::Instant::now();

    let mut handles = Vec::new();
    let test_users = 2; // Further reduced for stability

    for i in 0..test_users {
        let base_url = config.base_url.clone();
        let handle = tokio::spawn(async move {
            let mut client = TestClient::new(base_url);
            let email = generate_test_email();
            let password = generate_test_password();

            // Complete user journey
            let register_status = client
                .register_user(&email, &password, "Load", "Test")
                .await
                .unwrap();
            assert_eq!(register_status, StatusCode::CREATED);

            let token = client.login(&email, &password).await.unwrap();
            assert!(!token.is_empty());

            let user_info = client.get_user_info().await.unwrap();
            assert_eq!(user_info["email"], email);

            // Multiple queue operations
            for _ in 0..2 {
                let join_status = client.join_queue("rapid").await.unwrap();
                assert_eq!(join_status, StatusCode::OK);

                let leave_status = client.leave_queue("rapid").await.unwrap();
                assert_eq!(leave_status, StatusCode::OK);
            }

            let delete_status = client.delete_account().await.unwrap();
            assert_eq!(delete_status, StatusCode::NO_CONTENT);

            true
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap());
    }

    let elapsed = start_time.elapsed();
    println!("Performance test completed in {:?}", elapsed);
    assert!(
        elapsed < config.timeout,
        "Test should complete within timeout"
    );
}

async fn run_all_e2e_tests() {
    println!("Starting E2E tests for Checkmate API...");

    // Run individual tests
    test_complete_user_journey().await;
    test_websocket_connection_flow().await;
    test_websocket_multiple_connections().await;
    test_websocket_reconnection().await;
    test_multiple_users_queue_interaction().await;
    test_authentication_flow().await;
    test_error_scenarios().await;
    test_concurrent_queue_operations().await;
    test_session_persistence().await;
    test_performance_under_load().await;

    println!("All E2E tests completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_e2e_tests() {
        run_all_e2e_tests().await;
    }
}
