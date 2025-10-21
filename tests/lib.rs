// Test library for Checkmate
// This module contains all test utilities and configurations

pub mod config;
pub mod test_utils;
pub mod integration_tests;
pub mod e2e_tests;

// Re-export commonly used test utilities
pub use config::TestConfig;
pub use test_utils::*;

/// Test suite configuration and utilities
pub mod test_suite {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    /// Test suite runner
    pub struct TestSuite {
        config: TestConfig,
    }

    impl TestSuite {
        pub fn new() -> Self {
            Self {
                config: TestConfig::from_env(),
            }
        }

        pub fn with_config(config: TestConfig) -> Self {
            Self { config }
        }

        /// Run all tests in the suite
        pub async fn run_all(&self) -> TestResults {
            let mut results = TestResults::new();

            // Run unit tests
            if let Err(e) = self.run_unit_tests().await {
                eprintln!("Unit tests failed: {}", e);
                results.add_failure();
            } else {
                results.add_success();
            }

            // Run integration tests
            if let Err(e) = self.run_integration_tests().await {
                eprintln!("Integration tests failed: {}", e);
                results.add_failure();
            } else {
                results.add_success();
            }

            // Run E2E tests
            if let Err(e) = self.run_e2e_tests().await {
                eprintln!("E2E tests failed: {}", e);
                results.add_failure();
            } else {
                results.add_success();
            }

            results
        }

        async fn run_unit_tests(&self) -> Result<(), String> {
            // Unit tests are run via cargo test command
            // This is just a placeholder for test suite organization
            Ok(())
        }

        async fn run_integration_tests(&self) -> Result<(), String> {
            // Integration tests would be executed here
            // For now, this is a placeholder
            Ok(())
        }

        async fn run_e2e_tests(&self) -> Result<(), String> {
            // E2E tests would be executed here
            // For now, this is a placeholder
            Ok(())
        }
    }

    /// Test results tracker
    #[derive(Debug, Clone)]
    pub struct TestResults {
        pub total: usize,
        pub passed: usize,
        pub failed: usize,
        pub duration: Duration,
    }

    impl TestResults {
        pub fn new() -> Self {
            Self {
                total: 0,
                passed: 0,
                failed: 0,
                duration: Duration::default(),
            }
        }

        pub fn add_success(&mut self) {
            self.total += 1;
            self.passed += 1;
        }

        pub fn add_failure(&mut self) {
            self.total += 1;
            self.failed += 1;
        }

        pub fn success_rate(&self) -> f64 {
            if self.total == 0 {
                0.0
            } else {
                (self.passed as f64) / (self.total as f64) * 100.0
            }
        }

        pub fn is_successful(&self) -> bool {
            self.failed == 0
        }
    }

    impl Default for TestResults {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Test macros and utilities
#[macro_export]
macro_rules! assert_status {
    ($response:expr, $expected:expr) => {
        assert_eq!(
            $response.status().as_u16(),
            $expected,
            "Expected status code {}, got {}",
            $expected,
            $response.status().as_u16()
        )
    };
}

#[macro_export]
macro_rules! assert_json_contains {
    ($json:expr, $field:expr, $expected:expr) => {
        let value = $json
            .get($field)
            .unwrap_or_else(|| panic!("Field '{}' not found in JSON", $field));
        assert_eq!(value, $expected, "Field '{}' mismatch", $field);
    };
}

#[macro_export]
macro_rules! test_with_timeout {
    ($timeout:expr, $test:expr) => {
        tokio::time::timeout($timeout, $test)
            .await
            .expect("Test timed out")
    };
}

/// Common test fixtures
pub mod fixtures {
    use super::*;
    use checkmate::packages::shared::models::user::User;
    use uuid::Uuid;

    /// Create a test user with default values
    pub fn create_test_user() -> User {
        User::new(
            format!("test-{}@example.com", Uuid::new_v4()),
            "test-password".to_string(),
            "Test".to_string(),
            "User".to_string(),
        )
    }

    /// Create multiple test users
    pub fn create_test_users(count: usize) -> Vec<User> {
        (0..count)
            .map(|i| {
                User::new(
                    format!("test-user-{}@example.com", i),
                    "password123".to_string(),
                    format!("FirstName{}", i),
                    format!("LastName{}", i),
                )
            })
            .collect()
    }

    /// Test user credentials
    pub struct TestCredentials {
        pub email: String,
        pub password: String,
        pub first_name: String,
        pub last_name: String,
    }

    impl Default for TestCredentials {
        fn default() -> Self {
            Self {
                email: format!("test-{}@example.com", Uuid::new_v4()),
                password: "test-password-123".to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_results() {
        let mut results = test_suite::TestResults::new();
        assert_eq!(results.total, 0);
        assert_eq!(results.passed, 0);
        assert_eq!(results.failed, 0);
        assert_eq!(results.success_rate(), 0.0);
        assert!(results.is_successful());

        results.add_success();
        assert_eq!(results.total, 1);
        assert_eq!(results.passed, 1);
        assert_eq!(results.failed, 0);
        assert_eq!(results.success_rate(), 100.0);
        assert!(results.is_successful());

        results.add_failure();
        assert_eq!(results.total, 2);
        assert_eq!(results.passed, 1);
        assert_eq!(results.failed, 1);
        assert_eq!(results.success_rate(), 50.0);
        assert!(!results.is_successful());
    }

    #[test]
    fn test_fixtures() {
        let user = fixtures::create_test_user();
        assert!(user.email.contains("@example.com"));
        assert_eq!(user.password, "test-password");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
        assert_eq!(user.rating, 1200);

        let users = fixtures::create_test_users(3);
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].email, "test-user-0@example.com");
        assert_eq!(users[1].email, "test-user-1@example.com");
        assert_eq!(users[2].email, "test-user-2@example.com");
    }

    #[test]
    fn test_test_credentials() {
        let creds = fixtures::TestCredentials::default();
        assert!(creds.email.contains("@example.com"));
        assert_eq!(creds.password, "test-password-123");
        assert_eq!(creds.first_name, "Test");
        assert_eq!(creds.last_name, "User");
    }
}
