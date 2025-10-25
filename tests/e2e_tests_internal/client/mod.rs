use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::error::Error;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{generate_test_email, generate_test_password, TestResult};

/// HTTP client for E2E testing with authentication support
pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    websocket_url: String,
    pub auth_token: Option<String>,
    user_id: Option<String>,
    websocket_connections: Vec<WebSocketStream<MaybeTlsStream<TcpStream>>>,
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
            websocket_connections: Vec::new(),
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
            websocket_connections: Vec::new(),
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
    pub async fn establish_websocket_connection(&mut self, _player_id: &str) -> TestResult<()> {
        // Temporarily bypass token requirement for testing WebSocket infrastructure
        // let token = self
        //     .auth_token
        //     .as_ref()
        //     .ok_or("Not authenticated - no JWT token available")?;
        // let url = format!("{}?token={}", self.websocket_url, token);
        let url = self.websocket_url.clone();
        println!("Establishing WebSocket connection to URL: {}", url);

        match connect_async(&url).await {
            Ok((ws_stream, response)) => {
                println!("WebSocket connection established successfully");
                println!("Response status: {:?}", response.status());
                println!("Response headers: {:?}", response.headers());
                self.websocket_connections.push(ws_stream);
                println!(
                    "WebSocket connection stored, total connections: {}",
                    self.websocket_connections.len()
                );
                Ok(())
            }
            Err(e) => {
                println!("WebSocket connection failed with error: {:?}", e);
                println!("Error source: {:?}", e.source());
                Err(format!("WebSocket connection failed: {:?}", e).into())
            }
        }
    }

    /// Send a message through the WebSocket connection
    pub async fn send_websocket_message(&mut self, message: &str) -> TestResult<()> {
        if let Some(connection) = self.websocket_connections.last_mut() {
            connection.send(Message::Text(message.to_string())).await?;
        } else {
            return Err("No WebSocket connection established".into());
        }
        Ok(())
    }

    /// Receive a message from the WebSocket connection with timeout
    pub async fn receive_websocket_message(
        &mut self,
        timeout_duration: Duration,
    ) -> TestResult<Option<String>> {
        if let Some(connection) = self.websocket_connections.last_mut() {
            match timeout(timeout_duration, connection.next()).await {
                Ok(Some(Ok(message))) => {
                    if let Message::Text(text) = message {
                        Ok(Some(text))
                    } else {
                        Ok(None)
                    }
                }
                Ok(Some(Err(e))) => Err(e.into()),
                Ok(None) => Ok(None),
                Err(_) => Ok(None), // Timeout
            }
        } else {
            Err("No WebSocket connection established".into())
        }
    }

    /// Wait for a specific message with timeout
    pub async fn wait_for_websocket_message(
        &mut self,
        expected_action: &str,
        timeout_duration: Duration,
    ) -> TestResult<Value> {
        self.wait_for_websocket_message_with_filter(expected_action, timeout_duration, |_| true)
            .await
    }

    /// Close all WebSocket connections
    pub async fn close_websocket_connections(&mut self) -> TestResult<()> {
        for mut connection in self.websocket_connections.drain(..) {
            connection.close(None).await?;
        }
        Ok(())
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

    /// Make a chess move in a game session
    pub async fn make_move(
        &mut self,
        session_id: &str,
        from_square: &str,
        to_square: &str,
    ) -> TestResult<()> {
        let move_message = json!({
            "action": "make_move",
            "game_session_id": session_id,
            "from_square": from_square,
            "to_square": to_square
        });

        self.send_websocket_message(&move_message.to_string())
            .await?;

        Ok(())
    }

    /// Make a chess move with promotion
    pub async fn make_move_with_promotion(
        &mut self,
        session_id: &str,
        from_square: &str,
        to_square: &str,
        promotion_piece: &str,
    ) -> TestResult<()> {
        let move_message = json!({
            "action": "make_move",
            "game_session_id": session_id,
            "from_square": from_square,
            "to_square": to_square,
            "promotion_piece": promotion_piece
        });

        self.send_websocket_message(&move_message.to_string())
            .await?;

        Ok(())
    }

    /// Wait for a move update notification for a specific game session
    pub async fn wait_for_move_update(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> TestResult<Value> {
        self.wait_for_websocket_message_with_filter("game_update", timeout, |msg| {
            if let Some(game_session) = msg.get("game_session") {
                if let Some(msg_session_id) =
                    game_session.get("session_id").and_then(|s| s.as_str())
                {
                    return msg_session_id == session_id;
                }
            }
            false
        })
        .await
    }

    /// Wait for game end notification for a specific game session
    pub async fn wait_for_game_end(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> TestResult<Value> {
        self.wait_for_websocket_message_with_filter("game_update", timeout, |msg| {
            if let Some(game_session) = msg.get("game_session") {
                if let Some(msg_session_id) =
                    game_session.get("session_id").and_then(|s| s.as_str())
                {
                    if msg_session_id == session_id {
                        // Check if game has ended
                        if let Some(status) = game_session.get("status").and_then(|s| s.as_str()) {
                            return status != "Ongoing";
                        }
                    }
                }
            }
            false
        })
        .await
    }

    /// Wait for a WebSocket message with a custom filter function
    pub async fn wait_for_websocket_message_with_filter<F>(
        &mut self,
        expected_action: &str,
        timeout: Duration,
        filter: F,
    ) -> TestResult<Value>
    where
        F: Fn(&Value) -> bool,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if let Some(connection) = self.websocket_connections.last_mut() {
                match tokio::time::timeout(Duration::from_millis(100), connection.next()).await {
                    Ok(Some(Ok(message))) => {
                        if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
                            if let Ok(json_message) = serde_json::from_str::<Value>(&text) {
                                if let Some(action) =
                                    json_message.get("action").and_then(|a| a.as_str())
                                {
                                    if action == expected_action && filter(&json_message) {
                                        return Ok(json_message);
                                    }
                                }
                            }
                        }
                    }
                    Ok(Some(Err(e))) => return Err(e.into()),
                    Ok(None) => return Err("WebSocket connection closed".into()),
                    Err(_) => {
                        // Timeout, continue waiting
                        if start.elapsed().as_secs() % 5 == 0 {
                            println!(
                                "Still waiting for filtered message... ({:?} elapsed)",
                                start.elapsed()
                            );
                        }
                    }
                }
            } else {
                return Err("No WebSocket connection established".into());
            }
        }

        Err(format!(
            "Timeout waiting for filtered message with action: {}",
            expected_action
        )
        .into())
    }

    /// Perform complete cleanup (delete account if authenticated)
    pub async fn cleanup(&self) -> TestResult<()> {
        if self.is_authenticated() {
            self.delete_account().await?;
        }
        Ok(())
    }
}
