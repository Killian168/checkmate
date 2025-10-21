use std::time::Duration;

/// Configuration for E2E tests
#[derive(Debug, Clone)]
pub struct E2EConfig {
    /// Base URL for HTTP API endpoints
    pub base_url: String,
    /// WebSocket URL for real-time connections
    pub websocket_url: String,
    /// Maximum timeout for test operations
    pub timeout: Duration,
    /// Maximum number of concurrent users for load tests
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

/// Environment-specific configuration
impl E2EConfig {
    /// Create configuration for development environment
    pub fn development() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            websocket_url: "ws://localhost:8080".to_string(),
            timeout: Duration::from_secs(10),
            max_concurrent_users: 5,
        }
    }

    /// Create configuration for staging environment
    pub fn staging() -> Self {
        Self {
            base_url: "https://staging-api.example.com".to_string(),
            websocket_url: "wss://staging-ws.example.com".to_string(),
            timeout: Duration::from_secs(20),
            max_concurrent_users: 8,
        }
    }

    /// Create configuration for production environment
    pub fn production() -> Self {
        Self {
            base_url: "https://sq5wo33z30.execute-api.eu-west-1.amazonaws.com".to_string(),
            websocket_url: "wss://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev".to_string(),
            timeout: Duration::from_secs(30),
            max_concurrent_users: 10,
        }
    }
}
