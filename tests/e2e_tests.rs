//! End-to-End (E2E) tests for the Checkmate application
//!
//! This file serves as the main entry point for E2E tests, using the
//! organized module structure in the `e2e_tests_internal` directory.

// Import the organized E2E testing framework
mod e2e_tests_internal;

// Re-export for convenient access in tests
pub use e2e_tests_internal::{
    run_all_e2e_tests, run_e2e_tests_by_category, run_e2e_tests_with_config, E2EConfig, TestClient,
    TestResult, TestTimer, TestUser,
};

// Import test scenarios for direct access if needed

#[cfg(test)]
mod tests {
    use super::*;
    use e2e_tests_internal::scenarios::TestCategory;

    /// Complete user journey test - registration to deletion
    #[tokio::test]
    async fn run_complete_user_journey() {
        e2e_tests_internal::scenarios::authentication::test_complete_user_journey()
            .await
            .expect("Complete user journey test failed");
    }

    /// WebSocket connection flow test
    #[tokio::test]
    async fn run_websocket_connection_flow() {
        e2e_tests_internal::scenarios::websocket::test_websocket_connection_flow()
            .await
            .expect("WebSocket connection flow test failed");
    }

    /// Multiple WebSocket connections test
    #[tokio::test]
    async fn run_websocket_multiple_connections() {
        e2e_tests_internal::scenarios::websocket::test_websocket_multiple_connections()
            .await
            .expect("WebSocket multiple connections test failed");
    }

    /// WebSocket reconnection test
    #[tokio::test]
    async fn run_websocket_reconnection() {
        e2e_tests_internal::scenarios::websocket::test_websocket_reconnection()
            .await
            .expect("WebSocket reconnection test failed");
    }

    /// Multiple users queue interaction test
    #[tokio::test]
    async fn run_multiple_users_queue_interaction() {
        e2e_tests_internal::scenarios::queue::test_multiple_users_queue_interaction()
            .await
            .expect("Multiple users queue interaction test failed");
    }

    /// Authentication flow test
    #[tokio::test]
    async fn run_authentication_flow() {
        e2e_tests_internal::scenarios::authentication::test_authentication_flow()
            .await
            .expect("Authentication flow test failed");
    }

    /// Error scenarios test
    #[tokio::test]
    async fn run_error_scenarios() {
        e2e_tests_internal::scenarios::authentication::test_error_scenarios()
            .await
            .expect("Error scenarios test failed");
    }

    /// Concurrent queue operations test
    #[tokio::test]
    async fn run_concurrent_queue_operations() {
        e2e_tests_internal::scenarios::queue::test_concurrent_queue_operations()
            .await
            .expect("Concurrent queue operations test failed");
    }

    /// Session persistence test
    #[tokio::test]
    async fn run_session_persistence() {
        e2e_tests_internal::scenarios::authentication::test_session_persistence()
            .await
            .expect("Session persistence test failed");
    }

    /// Run all tests in a specific category
    #[tokio::test]
    async fn run_authentication_category_tests() {
        run_e2e_tests_by_category(TestCategory::Authentication)
            .await
            .expect("Authentication category tests failed");
    }

    /// Run all tests in queue category
    #[tokio::test]
    async fn run_queue_category_tests() {
        run_e2e_tests_by_category(TestCategory::Queue)
            .await
            .expect("Queue category tests failed");
    }

    /// Run all tests in WebSocket category
    #[tokio::test]
    async fn run_websocket_category_tests() {
        run_e2e_tests_by_category(TestCategory::WebSocket)
            .await
            .expect("WebSocket category tests failed");
    }

    /// Run all E2E tests (comprehensive test suite)
    #[tokio::test]
    async fn run_all_e2e_tests_suite() {
        run_all_e2e_tests()
            .await
            .expect("All E2E tests suite failed");
    }

    /// Test with custom configuration
    #[tokio::test]
    async fn run_tests_with_custom_config() {
        let config = E2EConfig::default();
        run_e2e_tests_with_config(config)
            .await
            .expect("Tests with custom configuration failed");
    }
}
