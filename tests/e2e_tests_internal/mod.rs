//! End-to-End (E2E) testing framework for the Checkmate application
//!
//! This module provides a comprehensive testing framework organized into logical components:
//! - `config`: Test configuration and environment setup
//! - `client`: HTTP and WebSocket client for API interactions
//! - `utils`: Test utilities and helper functions
//! - `scenarios`: Organized test scenarios by functionality

pub mod client;
pub mod config;
pub mod scenarios;
pub mod utils;

// Re-export commonly used items for convenient access
pub use client::TestClient;
pub use config::E2EConfig;
pub use utils::{TestResult, TestTimer, TestUser};

/// Run all E2E tests
///
/// This function executes all available E2E test scenarios and returns
/// a summary of the results. It's intended for use in integration tests
/// and CI/CD pipelines.
pub async fn run_all_e2e_tests() -> TestResult<()> {
    use scenarios::TestCategory;

    let category = TestCategory::All;
    println!("Running {} E2E tests...", category.test_count());

    // Authentication tests
    scenarios::authentication::test_complete_user_journey().await?;
    scenarios::authentication::test_authentication_flow().await?;
    scenarios::authentication::test_error_scenarios().await?;
    scenarios::authentication::test_session_persistence().await?;

    // Queue tests
    scenarios::queue::test_multiple_users_queue_interaction().await?;
    scenarios::queue::test_concurrent_queue_operations().await?;

    // WebSocket tests
    scenarios::websocket::test_websocket_connection_flow().await?;
    scenarios::websocket::test_websocket_multiple_connections().await?;
    scenarios::websocket::test_websocket_reconnection().await?;

    println!("All E2E tests completed successfully!");
    Ok(())
}

/// Run specific category of E2E tests
///
/// This function allows running tests from a specific category,
/// useful for targeted testing during development.
pub async fn run_e2e_tests_by_category(category: scenarios::TestCategory) -> TestResult<()> {
    println!(
        "Running {} E2E tests from category...",
        category.test_count()
    );

    match category {
        scenarios::TestCategory::Authentication => {
            scenarios::authentication::test_complete_user_journey().await?;
            scenarios::authentication::test_authentication_flow().await?;
            scenarios::authentication::test_error_scenarios().await?;
            scenarios::authentication::test_session_persistence().await?;
        }
        scenarios::TestCategory::Queue => {
            scenarios::queue::test_multiple_users_queue_interaction().await?;
            scenarios::queue::test_concurrent_queue_operations().await?;
        }
        scenarios::TestCategory::WebSocket => {
            scenarios::websocket::test_websocket_connection_flow().await?;
            scenarios::websocket::test_websocket_multiple_connections().await?;
            scenarios::websocket::test_websocket_reconnection().await?;
        }
        scenarios::TestCategory::All => {
            run_all_e2e_tests().await?;
        }
    }

    println!("Category E2E tests completed successfully!");
    Ok(())
}

/// E2E test runner with configuration
///
/// This function provides a more flexible way to run tests with custom
/// configuration, useful for testing against different environments.
pub async fn run_e2e_tests_with_config(config: E2EConfig) -> TestResult<()> {
    println!("Running E2E tests with custom configuration...");
    println!("Base URL: {}", config.base_url);
    println!("WebSocket URL: {}", config.websocket_url);
    println!("Timeout: {:?}", config.timeout);

    // Note: The individual test functions would need to be modified
    // to accept configuration parameters. For now, we run the standard tests.
    run_all_e2e_tests().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test runner that executes all E2E tests
    ///
    /// This test is marked as `ignore` by default since it requires
    /// external API endpoints to be available. Remove the `ignore`
    /// attribute when running against a live environment.
    #[tokio::test]
    #[ignore = "Requires external API endpoints"]
    async fn run_all_e2e_tests_integration() {
        let result = run_all_e2e_tests().await;
        assert!(result.is_ok(), "All E2E tests should pass");
    }
}
