use uuid::Uuid;

/// Test result wrapper for better error handling
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Generate a unique test email address
pub fn generate_test_email() -> String {
    format!("test-{}@example.com", Uuid::new_v4())
}

/// Generate a unique test password
pub fn generate_test_password() -> String {
    format!("password-{}", Uuid::new_v4())
}

/// Verify that the health endpoint is responding
pub async fn verify_health_endpoint(base_url: &str) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.get(&format!("{}/health", base_url)).send().await?;
    Ok(response.status() == reqwest::StatusCode::OK)
}

/// Test user credentials for authentication tests
#[derive(Debug, Clone)]
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

impl Default for TestUser {
    fn default() -> Self {
        Self {
            email: generate_test_email(),
            password: generate_test_password(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        }
    }
}

impl TestUser {
    /// Create a test user with custom details
    pub fn new(first_name: &str, last_name: &str) -> Self {
        Self {
            email: generate_test_email(),
            password: generate_test_password(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
        }
    }

    /// Create a test user with specific email
    pub fn with_email(email: &str) -> Self {
        Self {
            email: email.to_string(),
            password: generate_test_password(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        }
    }
}

/// Helper to measure test execution time
pub struct TestTimer {
    start_time: std::time::Instant,
}

impl TestTimer {
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn assert_within_timeout(&self, timeout: std::time::Duration) {
        assert!(
            self.elapsed() < timeout,
            "Test exceeded timeout of {:?}",
            timeout
        );
    }
}
