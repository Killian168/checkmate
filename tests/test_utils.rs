use aws_sdk_dynamodb::types::AttributeValue;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Test configuration for the test suite
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub base_url: String,
    pub jwt_secret: String,
    pub test_timeout_ms: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            jwt_secret: "test-secret-key".to_string(),
            test_timeout_ms: 5000,
        }
    }
}

/// Test user data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password: String,
    pub rating: i32,
}

impl TestUser {
    pub fn new(email: &str, password: &str, rating: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            password: password.to_string(),
            rating,
        }
    }

    pub fn to_dynamo_attributes(&self) -> HashMap<String, AttributeValue> {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), AttributeValue::S(self.id.clone()));
        attrs.insert("email".to_string(), AttributeValue::S(self.email.clone()));
        attrs.insert(
            "password".to_string(),
            AttributeValue::S(self.password.clone()),
        );
        attrs.insert(
            "rating".to_string(),
            AttributeValue::N(self.rating.to_string()),
        );
        attrs
    }
}

/// HTTP client wrapper for test requests
pub struct TestClient {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl TestClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
    }

    pub async fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()
            .await
    }

    pub async fn post_with_auth(
        &self,
        path: &str,
        body: &serde_json::Value,
        token: &str,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
    }
}

/// Mock repository for testing
pub struct MockRepository<T> {
    pub data: Arc<tokio::sync::RwLock<HashMap<String, T>>>,
}

impl<T> MockRepository<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, key: String, value: T) {
        let mut data = self.data.write().await;
        data.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    pub async fn remove(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let mut data = self.data.write().await;
        data.remove(key)
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
}

/// Test assertions utilities
pub mod assertions {
    use super::*;

    pub fn assert_status_code(response: &reqwest::Response, expected: u16) {
        assert_eq!(
            response.status().as_u16(),
            expected,
            "Expected status code {}, got {}",
            expected,
            response.status().as_u16()
        );
    }

    pub async fn assert_json_response<T>(response: reqwest::Response) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        let text = response.text().await.expect("Failed to read response text");
        serde_json::from_str(&text).expect("Failed to parse JSON response")
    }

    pub fn assert_error_message(response_json: &serde_json::Value, expected_message: &str) {
        let error_message = response_json
            .get("error")
            .or_else(|| response_json.get("message"))
            .and_then(|v| v.as_str())
            .expect("No error message found in response");

        assert_eq!(
            error_message, expected_message,
            "Expected error message '{}', got '{}'",
            expected_message, error_message
        );
    }
}

/// Test setup and teardown utilities
pub mod setup {
    use super::*;

    pub async fn setup_test_env() -> TestConfig {
        // Set up test environment variables
        std::env::set_var("JWT_SECRET", "test-secret-key");
        std::env::set_var("USERS_TABLE", "test-users-table");
        std::env::set_var("QUEUE_TABLE", "test-queue-table");

        TestConfig::default()
    }

    pub async fn cleanup_test_env() {
        // Clean up test environment variables
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("USERS_TABLE");
        std::env::remove_var("QUEUE_TABLE");
    }
}

/// Test data generators
pub mod generators {
    use super::*;

    pub fn generate_test_email() -> String {
        format!("test-{}@example.com", Uuid::new_v4())
    }

    pub fn generate_test_password() -> String {
        format!("password-{}", Uuid::new_v4())
    }

    pub async fn verify_health_endpoint(base_url: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client.get(&format!("{}/health", base_url)).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    pub fn generate_test_user() -> TestUser {
        let id = Uuid::new_v4().to_string();
        TestUser {
            id,
            email: format!("test-{}@example.com", Uuid::new_v4()),
            password: "test-password".to_string(),
            rating: 1200,
        }
    }

    pub fn generate_jwt_token(user_id: &str) -> String {
        // Simple mock token generation for testing
        format!("mock-token-{}", user_id)
    }

    pub fn generate_queue_entry(player_id: &str, rating: i32) -> HashMap<String, AttributeValue> {
        let mut attrs = HashMap::new();
        attrs.insert(
            "player_id".to_string(),
            AttributeValue::S(player_id.to_string()),
        );
        attrs.insert(
            "queue_rating".to_string(),
            AttributeValue::S(format!("rating_{}", rating)),
        );
        attrs.insert(
            "joined_at".to_string(),
            AttributeValue::S(chrono::Utc::now().to_rfc3339()),
        );
        attrs
    }
}
