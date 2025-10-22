pub mod client;
pub mod config;
pub mod scenarios;
pub mod utils;

pub use client::TestClient;
pub use config::E2EConfig;
pub use utils::{TestResult, TestTimer, TestUser};

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
    }

    println!("Category E2E tests completed successfully!");
    Ok(())
}
