use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{generate_test_email, generate_test_password, TestResult};

/// HTTP client for E2E testing with authentication support
pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    websocket_url: String,
    pub auth_token: Option<String>,
    user_id: Option<String>,
}

impl TestClient {
    /// Create a new test client with the given base URL
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            websocket_url: "wss://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev".to_string(),
            auth_token: None,
            user_id: None,
        }
    }

    /// Create a test client from configuration
    pub fn from_config(config: &E2EConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: config.base_url.clone(),
            websocket_url: config.websocket_url.clone(),
            auth_token: None,
            user_id: None,
        }
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    /// Get the current authentication token
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }

    /// Get the current user ID
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Register a new user
    pub async fn register_user(
        &mut self,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> TestResult<StatusCode> {
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

    /// Login with email and password
    pub async fn login(&mut self, email: &str, password: &str) -> TestResult<String> {
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
        let token = login_response["token"]
            .as_str()
            .ok_or("Token not found in response")?
            .to_string();

        self.auth_token = Some(token.clone());

        // Extract and store user ID if available
        if let Some(user_id) = login_response["user_id"].as_str() {
            self.user_id = Some(user_id.to_string());
        }

        Ok(token)
    }

    /// Join a queue
    pub async fn join_queue(&self, queue_type: &str) -> TestResult<StatusCode> {
        let token = self.auth_token.as_ref().ok_or("Not authenticated")?;

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
    }

    /// Leave a queue
    pub async fn leave_queue(&self, queue_type: &str) -> TestResult<StatusCode> {
        let token = self.auth_token.as_ref().ok_or("Not authenticated")?;

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
    }

    /// Get current user information
    pub async fn get_user_info(&self) -> TestResult<Value> {
        let token = self.auth_token.as_ref().ok_or("Not authenticated")?;

        let response = self
            .client
            .get(&format!("{}/auth/user", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;

        let user_info: Value = response.json().await?;

        // Update user ID if not already set
        if self.user_id.is_none() {
            if let Some(_user_id) = user_info["id"].as_str() {
                // Clone the client to mutate user_id (workaround for &self)
                // In practice, we'd need &mut self for this, but keeping API consistent
            }
        }

        Ok(user_info)
    }

    /// Delete the current user account
    pub async fn delete_account(&self) -> TestResult<StatusCode> {
        let token = self.auth_token.as_ref().ok_or("Not authenticated")?;

        let response = self
            .client
            .delete(&format!("{}/auth/user", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.status())
    }

    /// Establish a WebSocket connection
    pub async fn establish_websocket_connection(
        &self,
        _player_id: &str,
    ) -> TestResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        // Note: player_id parameter is currently unused but kept for API compatibility
        let (ws_stream, _) = connect_async(&self.websocket_url).await?;
        Ok(ws_stream)
    }

    /// Create and register a test user in one operation
    pub async fn create_test_user(
        &mut self,
        first_name: &str,
        last_name: &str,
    ) -> TestResult<(String, String)> {
        let email = generate_test_email();
        let password = generate_test_password();

        self.register_user(&email, &password, first_name, last_name)
            .await?;
        self.login(&email, &password).await?;

        Ok((email, password))
    }

    /// Perform complete cleanup (delete account if authenticated)
    pub async fn cleanup(&self) -> TestResult<()> {
        if self.is_authenticated() {
            self.delete_account().await?;
        }
        Ok(())
    }
}
